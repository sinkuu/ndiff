use ndarray::{Array2, Axis};
use std::num::NonZeroUsize;
use circular_queue::CircularQueue;

#[derive(Clone, PartialEq, Eq)]
enum BareOp {
    Del,
    Ins,
    Nop,
}

#[derive(Debug)]
enum Op<T> {
    Del(T),
    Ins(T),
    Nop(NonZeroUsize),
}

fn opsel_min(a: usize, b: usize, c: usize) -> (BareOp, usize) {
    if a <= b && a <= c {
        return (BareOp::Del, a);
    }
    if b <= c {
        return (BareOp::Ins, b);
    }

    (BareOp::Nop, c)
}

fn diff_ops<T: Copy + PartialEq>(
    a: &[T],
    b: &[T],
    eq: impl Fn(&T, &T) -> bool,
    filter_edit: impl Fn(&T) -> bool,
) -> Vec<Op<T>> {
    let shape = (a.len() + 1, b.len() + 1);
    let mut cost = Array2::zeros(shape);
    let mut dir = Array2::from_elem(shape, BareOp::Nop);
    for (i, (e, d)) in cost
        .index_axis_mut(Axis(0), 0)
        .into_iter()
        .zip(dir.index_axis_mut(Axis(0), 0))
        .enumerate()
    {
        *e = i;
        *d = BareOp::Ins;
    }
    for (i, (e, d)) in cost
        .index_axis_mut(Axis(1), 0)
        .into_iter()
        .zip(dir.index_axis_mut(Axis(1), 0))
        .enumerate()
    {
        *e = i;
        *d = BareOp::Del;
    }
    dir[[0, 0]] = BareOp::Nop;
    for (i, a) in a.iter().enumerate() {
        for (j, b) in b.iter().enumerate() {
            let (i, j) = (i + 1, j + 1);

            let (k, x) = if eq(a, b) {
                opsel_min(
                    cost[[i - 1, j]] + 1,
                    cost[[i, j - 1]] + 1,
                    cost[[i - 1, j - 1]],
                )
            } else {
                opsel_min(cost[[i - 1, j]] + 1, cost[[i, j - 1]] + 1, usize::MAX)
            };

            dir[[i, j]] = k;
            cost[[i, j]] = x;
        }
    }

    let (mut i, mut j) = (a.len(), b.len());
    let mut ops = vec![];
    let push_nop = |ops: &mut Vec<Op<T>>| {
        if let Some(Op::Nop(count)) = ops.last_mut() {
            *count = NonZeroUsize::new(count.get() + 1).unwrap();
        } else {
            ops.push(Op::Nop(NonZeroUsize::new(1).unwrap()));
        }
    };
    while (i, j) != (0, 0) {
        match dir[[i, j]] {
            BareOp::Nop => {
                i -= 1;
                j -= 1;
                push_nop(&mut ops);
            }
            BareOp::Ins => {
                j -= 1;
                if filter_edit(&b[j]) {
                    ops.push(Op::Ins(b[j]));
                } else {
                    push_nop(&mut ops);
                }
            }
            BareOp::Del => {
                i -= 1;
                if filter_edit(&a[i]) {
                    ops.push(Op::Del(a[i]));
                } else {
                    push_nop(&mut ops);
                }
            }
        }
    }
    ops.reverse();
    ops
}

fn diff(a: &[&str], b: &[&str]) {
    let mut ops = diff_ops(
        &a,
        &b,
        |a, b| {
            diff_ops(
                &a.chars().collect::<Vec<_>>(),
                &b.chars().collect::<Vec<_>>(),
                PartialEq::eq,
                |a| !a.is_numeric(),
            )
            .iter()
            .all(|op| matches!(op, Op::Nop(_)))
        },
        |a| !a.chars().all(|c| c.is_numeric()),
    );

    let mut buf = CircularQueue::with_capacity(6);
    let print_buf = |buf: &mut CircularQueue<&str>| {
        for b in buf.asc_iter() {
            println!("  {}", b);
        }
        buf.clear();
    };
    let (mut a, mut b) = (a.iter(), b.iter());
    for op in ops {
        print_buf(&mut buf);
        match op {
            Op::Nop(count) => {
                let n = count.get() - 1;
                for l in a.by_ref().take(n + 1) {
                    buf.push(l);
                }
                b.nth(n).unwrap();
            }
            Op::Ins(_) => {
                println!("{}+ {}{}", termion::color::Fg(termion::color::Green), b.next().unwrap(), termion::style::Reset);
            }
            Op::Del(_) => {
                println!("{}- {}{}", termion::color::Fg(termion::color::Red), a.next().unwrap(), termion::style::Reset);
            }
        }
    }
    print_buf(&mut buf);
    assert!(matches!((a.next(), b.next()), (None, None)));
}

fn main() {
}
