import argparse, base64, json
import pyarrow as pa
import pyarrow.ipc as ipc
from pyspark.sql.connect.proto import relations_pb2


def lit_value(expr):
    if expr.HasField("literal"):
        lit = expr.literal
        if lit.HasField("long"):
            return lit.long
        if lit.HasField("integer"):
            return lit.integer
        if lit.HasField("short"):
            return lit.short
        if lit.HasField("byte"):
            return lit.byte
    return None


def eval_expr(expr, row):
    if expr.HasField("unresolved_attribute"):
        name = expr.unresolved_attribute.unparsed_identifier
        return row[name]
    if expr.HasField("literal"):
        return lit_value(expr)
    if expr.HasField("unresolved_function"):
        fn = expr.unresolved_function
        name = fn.function_name.lower()
        args = [eval_expr(a, row) for a in fn.arguments]
        if name in {"+", "add"} and len(args) == 2:
            return args[0] + args[1]
    if expr.HasField("cast"):
        return eval_expr(expr.cast.expr, row)
    if expr.HasField("sort_order"):
        return eval_expr(expr.sort_order.child, row)
    if expr.HasField("alias"):
        return eval_expr(expr.alias.expr, row)
    if expr.HasField("expression_string"):
        e = expr.expression_string.expression.strip().lower()
        if e == "id":
            return row["id"]
        if e.endswith(" as id") and "id +" in e:
            # Handle common spark.selectExpr("id + 1 as id") shape.
            rhs = e.split("id +", 1)[1].split(" as ", 1)[0].strip()
            return row["id"] + int(rhs)
    raise ValueError(f"unsupported expression: {expr.WhichOneof('expr_type')}")


def expr_name(expr):
    if expr.HasField("alias") and expr.alias.name:
        return expr.alias.name[0]
    if expr.HasField("unresolved_attribute"):
        return expr.unresolved_attribute.unparsed_identifier
    if expr.HasField("unresolved_function"):
        return expr.unresolved_function.function_name
    if expr.HasField("expression_string"):
        text = expr.expression_string.expression.strip()
        if " as " in text.lower():
            return text.rsplit(" ", 1)[-1]
        return text
    return "col"




def extract_count_expr(expr):
    if expr.HasField("unresolved_function"):
        return expr.unresolved_function.function_name.lower() == "count"
    if expr.HasField("alias"):
        return extract_count_expr(expr.alias.expr)
    if expr.HasField("cast"):
        return extract_count_expr(expr.cast.expr)
    if expr.HasField("expression_string"):
        return "count(" in expr.expression_string.expression.lower()
    return False


def format_show_string(rows, num_rows, truncate):
    display = rows[:num_rows]
    cols = list(rows[0].keys()) if rows else []
    widths = {c: len(c) for c in cols}
    for row in display:
        for c in cols:
            v = "null" if row.get(c) is None else str(row.get(c))
            if truncate and truncate > 0 and len(v) > truncate:
                v = v[: max(0, truncate - 3)] + "..."
            widths[c] = max(widths[c], len(v))

    def sep():
        return "+" + "+".join("-" * (widths[c] + 2) for c in cols) + "+"

    out = []
    if cols:
        out.append(sep())
        out.append("| " + " | ".join(c.ljust(widths[c]) for c in cols) + " |")
        out.append(sep())
        for row in display:
            vals = []
            for c in cols:
                v = "null" if row.get(c) is None else str(row.get(c))
                if truncate and truncate > 0 and len(v) > truncate:
                    v = v[: max(0, truncate - 3)] + "..."
                vals.append(v.ljust(widths[c]))
            out.append("| " + " | ".join(vals) + " |")
        out.append(sep())
    if len(rows) > num_rows:
        out.append(f"only showing top {num_rows} rows")
    return "\n".join(out)

def eval_relation(rel):
    rel_type = rel.WhichOneof("rel_type")
    if rel.HasField("range"):
        r = rel.range
        step = r.step if r.step else 1
        data = [{"id": i} for i in range(r.start, r.end, step)]
        return data
    if rel.HasField("repartition"):
        return eval_relation(rel.repartition.input)
    if rel.HasField("show_string"):
        src = eval_relation(rel.show_string.input)
        rendered = format_show_string(src, rel.show_string.num_rows or 20, rel.show_string.truncate)
        return [{"show_string": rendered}]
    if rel.HasField("to_df"):
        return eval_relation(rel.to_df.input)
    if rel.HasField("subquery_alias"):
        return eval_relation(rel.subquery_alias.input)
    if rel.HasField("limit"):
        src = eval_relation(rel.limit.input)
        return src[: rel.limit.limit]
    if rel.HasField("sort"):
        return eval_relation(rel.sort.input)
    if rel.HasField("with_columns_renamed"):
        return eval_relation(rel.with_columns_renamed.input)
    if rel.HasField("hint"):
        return eval_relation(rel.hint.input)
    if rel.HasField("project"):
        src = eval_relation(rel.project.input)
        out = []
        for row in src:
            nr = {}
            for e in rel.project.expressions:
                nr[expr_name(e)] = eval_expr(e, row)
            out.append(nr)
        return out
    if rel.HasField("aggregate"):
        src = eval_relation(rel.aggregate.input)
        out_row = {}
        for e in rel.aggregate.aggregate_expressions:
            if extract_count_expr(e):
                out_row[expr_name(e)] = len(src)
                continue
            out_row[expr_name(e)] = None
        return [out_row]
    raise ValueError(f"unsupported relation: {rel_type}")


def to_arrow(rows):
    if not rows:
        rows = [{"empty": None}]
    cols = {k: [r.get(k) for r in rows] for k in rows[0].keys()}
    table = pa.table(cols)
    sink = pa.BufferOutputStream()
    with ipc.RecordBatchStreamWriter(sink, table.schema) as writer:
        writer.write_table(table)
    return sink.getvalue().to_pybytes(), table.schema


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root-b64", required=True)
    args = ap.parse_args()
    rel = relations_pb2.Relation()
    rel.ParseFromString(base64.b64decode(args.root_b64))
    rows = eval_relation(rel)
    data, schema = to_arrow(rows)
    print(json.dumps({
        "arrow_b64": base64.b64encode(data).decode(),
        "row_count": len(rows),
        "fields": [{"name": f.name, "dtype": str(f.type)} for f in schema]
    }))


if __name__ == "__main__":
    main()
