from fractions import Fraction
import itertools
import copy
from main import simpler_interval, apotome, interval_to_cents

prepare = open("prepare", "r")

interval_map = dict()

for line in prepare:
    parts = list(map(str.strip, line.split("-")))
    interval = Fraction(parts[0])
    cents = eval(parts[1])
    tunings = eval(parts[2])

    interval_map[interval] = (interval, [interval], cents, tunings)

iterator = list(copy.deepcopy(interval_map).items())

# for (interval1, e1), (interval2, e2) in itertools.product(iterator, iterator):
# for (interval1, e1), (interval2, e2) in itertools.product(iterator, iterator):
for i in range(len(iterator)):
    for j in range(i + 1, len(iterator)):
        (interval1, (i1, l1, c1, tunings1)) = iterator[i]
        (interval2, (i2, l2, c2, tunings2)) = iterator[j]
        # print(interval1, e1, interval2, e2)
        # print(interval1, interval2)
        interval = interval2 / interval1

        if simpler_interval(interval, apotome) == interval:
            interval_map[interval] = (
                interval,
                [interval1, interval2],
                interval_to_cents(interval),
                list(itertools.product(tunings1, tunings2)),
            )
    # if interval1 != interval2:
    #     print(interval1 / interval2)

for i, e in interval_map.items():
    print(i, " - ", e)
