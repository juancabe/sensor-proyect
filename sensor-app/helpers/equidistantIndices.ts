// Given an array A `len`, produce number array B where B.len = `max` and contains A's indices equally distributed
// -- Example
//     0  1  2  3  4  5  6
// A: [a, b, c, d, e, f, g]
//
//              | equidistantIndices(A, 12)
//              V
// B: [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5]
//     a  a  b  b  c  c  d  d  e  e  f  f
//
export function equidistantIndices(len: number, max: number): number[] {
    console.log('equidistantIndices: len ', len, ', max ', max);
    if (max <= 0 || len <= 0) {
        return [];
    }

    if (len === 1) {
        // Si len es 1 siempre sera [0]
        return [0];
    }

    let indices: number[];

    if (max === 1) {
        // Central item (center + 1 if N is pair)
        indices = [Math.floor(len / 2)];
    } else {
        // Max indices === len
        max = Math.min(max, len);
        // Proportional spacing between the range [0, N-1]
        const step = (len - 1) / (max - 1);
        indices = Array.from({ length: max }, (_, i) => Math.round(i * step));
    }

    console.log('equidistantIndices: indices', indices);

    return indices;
}
