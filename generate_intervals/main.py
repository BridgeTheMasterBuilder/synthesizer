import math
from fractions import Fraction

nothing = Fraction(0, 1)
unison = Fraction(1, 1)
apotome = Fraction(2187, 2048)
factors = [3, 5, 7, 11, 13, 17, 19, 23, 29, 31]
category_names = [
    "Unison",
    "Minor 2nd",
    "Major 2nd",
    "Minor 3rd",
    "Major 3rd",
    "Perfect 4th",
    "Augmented 4th",
    "Perfect 5th",
    "Minor 6th",
    "Major 6th",
    "Minor 7th",
    "Major 7th",
]


def powerset(L):
    result = [[]]
    for e in L:
        result.extend([subset + [e] for subset in result])
    return result


tunings = powerset(factors)


def simpler_interval(interval1, interval2):
    min_numerator = min(interval1.numerator, interval2.numerator)
    min_denominator = min(interval1.denominator, interval2.denominator)

    if (
        min_numerator == interval1.numerator
        and min_denominator == interval1.denominator
    ):
        return interval1
    elif (
        min_numerator == interval1.numerator
        and min_denominator == interval2.denominator
    ):
        return interval1
    elif (
        min_numerator == interval2.numerator
        and min_denominator == interval1.denominator
    ):
        return interval2
    else:
        return interval2


def octave_reduce(interval):
    while interval < Fraction(1, 1):
        interval *= 2

    while interval >= Fraction(2, 1):
        interval /= 2

    return interval


def generate_intervals_with_factors(intervals, factors):
    new_intervals = set()

    for factor in factors:
        for interval1 in intervals:
            for interval2 in intervals:
                new_ascending_interval = octave_reduce(interval1 * factor)
                new_descending_interval = octave_reduce(interval1 / factor)

                new_intervals.add(new_ascending_interval)
                new_intervals.add(new_descending_interval)

                bigger = max(interval1, interval2)
                smaller = min(interval1, interval2)

                difference = octave_reduce(bigger / smaller)

                new_intervals.add(difference)

    if len(new_intervals) == 0 or len(intervals) >= 50:
        return intervals
    else:
        intervals.update(new_intervals)

        return generate_intervals_with_factors(intervals, factors)


def interval_to_cents(interval):
    if interval == 0:
        return 0.0
    else:
        return math.log2(interval) * 1200


def partition_intervals(intervals):
    unis = set()
    m2 = set()
    M2 = set()
    m3 = set()
    M3 = set()
    P4 = set()
    A4 = set()
    P5 = set()
    m6 = set()
    M6 = set()
    m7 = set()
    M7 = set()

    for interval in intervals:
        cents = interval_to_cents(interval)

        if 1150 <= cents < 1200 or cents < 50:
            unis.add(interval)
        elif 50 <= cents < 150:
            m2.add(interval)
        elif 150 <= cents < 250:
            M2.add(interval)
        elif 250 <= cents < 350:
            m3.add(interval)
        elif 350 <= cents < 450:
            M3.add(interval)
        elif 450 <= cents < 550:
            P4.add(interval)
        elif 550 <= cents < 650:
            A4.add(interval)
        elif 650 <= cents < 750:
            P5.add(interval)
        elif 750 <= cents < 850:
            m6.add(interval)
        elif 850 <= cents < 950:
            M6.add(interval)
        elif 950 <= cents < 1050:
            m7.add(interval)
        elif 1050 <= cents < 1150:
            M7.add(interval)

    return list(
        map(
            lambda category: {nothing} if len(category) == 0 else category,
            [unis, m2, M2, m3, M3, P4, A4, P5, m6, M6, m7, M7],
        )
    )


def simplest_interval(intervals):
    simplest_interval = apotome

    for interval in intervals:
        if simpler_interval(simplest_interval, interval) == interval:
            simplest_interval = interval

    return simplest_interval


def generate_intervals(tunings):
    tables = open("../tables.rs", "w")
    table = open("table", "w")

    interval_map = dict()

    tables.write(f"pub const TABLES: [[f64; 12]; {len(tunings)}] = [\n")
    for factors in tunings:
        table.write(f"Prime factors: {sorted(factors)}\n")

        intervals = generate_intervals_with_factors({unison}, factors)
        intervals = filter(lambda x: simpler_interval(x, apotome) == x, intervals)
        categories = partition_intervals(intervals)

        tables.write("\t[\n")
        for idx, category in enumerate(categories):
            if len(category) != 0:
                interval = simplest_interval(category)

                if interval not in interval_map:
                    interval_map[interval] = {frozenset(factors)}
                else:
                    interval_map[interval].add(frozenset(factors))

                num, denom = interval.as_integer_ratio()
                tables.write(f"\t\t{num}.0/{denom}.0,\n")
                table.write(
                    f"{category_names[idx]}: {interval} ({interval_to_cents(interval)})\n"
                )

        tables.write("\t],\n")
        table.write("\n")
    tables.write("];\n")

    for interval_entry in sorted(interval_map.items()):
        interval, tunings = interval_entry
        num, denom = interval.as_integer_ratio()
        if num != 0 and interval != unison:
            print(f"{num:4}/{denom:<4} - {interval_to_cents(interval):18}", end=" - ")
            for tuning in tunings:
                print("{", end=" ")
                for factor in tuning:
                    print(factor, end=" ")
                print("}", end=" ")
            print()

    return interval_map
