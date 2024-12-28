import math

intervals = [((8192 * 6) / 31) * x for x in range(1, 32)]


def distr(x):
    n = math.floor(x / 4096)
    k = x % 4096

    if k > (4096 / 2):
        return n + 1, k - 4096
    else:
        return n, k


def foo(x):
    (n, p) = x
    p2 = round(8192 + p)
    res = (p2 % 128, math.floor(p2 / 128))

    return n, res


distribution = map(distr, intervals)
pitch_bend_values = map(foo, distribution)
# print(list(distribution))
print(*list(pitch_bend_values), sep='\n')
