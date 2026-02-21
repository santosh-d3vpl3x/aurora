from pyspark.sql import SparkSession


def check(label, fn, expected):
    print(f"running {label}", flush=True)
    actual = fn()
    if actual != expected:
        raise AssertionError(f"{label}: expected {expected!r}, got {actual!r}")


def collect_first_ids(df, n):
    return [int(r[0]) for r in df.limit(n).collect()]


def main():
    spark = SparkSession.builder.remote("sc://127.0.0.1:15002").getOrCreate()
    total = 0

    for idx, (start, end, step) in enumerate([
        (0, 0, 1), (0, 1, 1), (0, 2, 1), (0, 10, 1), (5, 15, 1),
        (10, 20, 2), (0, 100, 5), (3, 30, 3), (100, 110, 1), (-5, 5, 1),
        (-10, 10, 2), (1, 50, 7), (50, 51, 1), (8, 9, 1), (0, 1000, 100),
        (7, 37, 5), (9, 99, 9), (2, 3, 1), (20, 20, 1), (20, 21, 1),
    ], start=1):
        expected = len(list(range(start, end, step)))
        check(f"{idx:02d} range.count", lambda s=start, e=end, st=step: spark.range(s, e, st).count(), expected)
        total += 1

    repartition_cases = [
        (0, 100, 1, 1), (0, 100, 1, 2), (0, 100, 1, 8), (5, 55, 5, 3), (10, 110, 10, 4),
        (-10, 10, 1, 6), (0, 1000, 3, 7), (3, 300, 3, 9), (0, 64, 2, 5), (1, 65, 2, 11),
    ]
    for offset, (start, end, step, parts) in enumerate(repartition_cases, start=21):
        expected = len(list(range(start, end, step)))
        check(f"{offset:02d} repartition.count", lambda s=start, e=end, st=step, p=parts: spark.range(s, e, st).repartition(p).count(), expected)
        total += 1

    select_cases = [(10,0),(10,1),(10,2),(10,3),(10,4),(10,5),(20,1),(20,10),(50,7),(100,9)]
    for offset, (n, inc) in enumerate(select_cases, start=31):
        check(f"{offset:02d} selectExpr.count", lambda n=n, inc=inc: spark.range(n).selectExpr(f"id + {inc} as id").count(), n)
        total += 1

    check("41 limit.count", lambda: spark.range(100).limit(10).count(), 10); total += 1
    check("42 sort.count", lambda: spark.range(25).sort("id").count(), 25); total += 1
    check("43 hint.count", lambda: spark.range(30).hint("broadcast").count(), 30); total += 1
    check("44 withColumnRenamed.count", lambda: spark.range(12).withColumnRenamed("id", "x").count(), 12); total += 1
    check("45 toDF.count", lambda: spark.range(11).toDF("x").count(), 11); total += 1

    check("46 first ids", lambda: collect_first_ids(spark.range(5), 3), [0, 1, 2]); total += 1
    check("47 shifted ids", lambda: collect_first_ids(spark.range(5).selectExpr("id + 1 as id"), 3), [1, 2, 3]); total += 1
    check("48 limit ids", lambda: collect_first_ids(spark.range(10).limit(2), 2), [0, 1]); total += 1

    check("49 show header", lambda: ("id" in spark.range(5)._show_string(20, 20, False) and "+" in spark.range(5)._show_string(20,20,False)), True); total += 1
    check("50 show shifted", lambda: ("| 1" in spark.range(5).selectExpr("id + 1 as id")._show_string(20, 20, False)), True); total += 1

    print(f"validated {total} spark.range checks")


if __name__ == "__main__":
    main()
