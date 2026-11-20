use gpui::TestAppContext;

use crate::cell::Guarded;

#[gpui::test]
fn jobs_run_in_the_order_they_were_queued(cx: &mut TestAppContext) {
    let cell = Guarded::new(Vec::<u32>::new(), &cx.executor());

    let results: Vec<_> = (0..20)
        .map(|value| {
            cell.spawn(move |values| {
                values.push(value);
                values.len()
            })
        })
        .collect();
    cx.run_until_parked();

    for (index, result) in results.into_iter().enumerate() {
        assert_eq!(cx.foreground_executor().block_test(result), Ok(index + 1));
    }
    assert_eq!(
        cell.peek(|values| values.clone()),
        Some((0..20).collect::<Vec<u32>>())
    );
}

#[gpui::test]
fn edits_made_while_a_job_runs_wait_for_it(cx: &mut TestAppContext) {
    let cell = Guarded::new(Vec::<u32>::new(), &cx.executor());
    let inside = cell.clone();

    assert_eq!(
        cell.edit(|values| values.push(0), |values| values.len()),
        Some(1)
    );
    let job = cell.spawn(move |values| {
        values.push(1);
        assert_eq!(
            inside.edit(|values| values.push(3), |values| values.len()),
            None
        );
        assert_eq!(inside.peek(|values| values.len()), None);
        assert_eq!(inside.take(|values| values.len()), None);
        values.push(2);
    });
    cx.run_until_parked();

    assert_eq!(cx.foreground_executor().block_test(job), Ok(()));
    assert_eq!(cell.peek(|values| values.clone()), Some(vec![0, 1, 2, 3]));
}

#[gpui::test]
fn queued_edits_are_applied_before_the_next_job(cx: &mut TestAppContext) {
    let cell = Guarded::new(Vec::<u32>::new(), &cx.executor());
    let inside = cell.clone();

    let first = cell.spawn(move |values| {
        inside.edit(|values| values.push(2), |_| ());
        values.push(1);
    });
    let second = cell.spawn(|values| values.clone());
    cx.run_until_parked();

    assert_eq!(cx.foreground_executor().block_test(first), Ok(()));
    assert_eq!(cx.foreground_executor().block_test(second), Ok(vec![1, 2]));
}

#[gpui::test]
fn a_panicking_job_does_not_stop_the_queue(cx: &mut TestAppContext) {
    let cell = Guarded::new(0u32, &cx.executor());

    let broken = cell.spawn(|value| {
        *value += 1;
        assert_eq!(*value, 0);
    });
    let next = cell.spawn(|value| {
        *value += 1;
        *value
    });
    let settled = cell.settled();
    cx.run_until_parked();

    assert!(cx.foreground_executor().block_test(broken).is_err());
    assert_eq!(cx.foreground_executor().block_test(next), Ok(2));
    assert_eq!(cx.foreground_executor().block_test(settled), Ok(()));
    assert_eq!(cell.peek(|value| *value), Some(2));
}
