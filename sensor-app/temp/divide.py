def equidistant_indices(N, M, one_based=False):
    if M <= 0:
        return []
    if M == 1:
        idx = [(N)//2]
    else:
        idx = [round(i*(N-1)/(M-1)) for i in range(M)]
    return [i+1 for i in idx] if one_based else idx

mul = 4
max = 1.1

for i in range(0,13):
    n = i * mul
    m = int(n * max)
    print("for n:", n, ", m", m, ": ", equidistant_indices(n, m))

