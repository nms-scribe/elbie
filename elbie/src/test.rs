
#[test]
fn test_bags() {
    use super::Bag;

    let mut bag_a = Bag::new();
    _ = bag_a.insert(1);
    _ = bag_a.insert(2);
    _ = bag_a.insert(3);
    _ = bag_a.insert(4);

    let mut bag_b = Bag::new();
    _ = bag_b.insert(2);
    _ = bag_b.insert(4);
    _ = bag_b.insert(6);

    let bag_union1 = bag_a.union(&bag_b);
    assert_eq!(bag_union1.list(),vec![1,2,3,4,6]);
    let bag_union2 = bag_b.union(&bag_a);
    assert_eq!(bag_union2.list(),vec![1,2,3,4,6]);
    let bag_intersection1 = bag_a.intersection(&bag_b);
    assert_eq!(bag_intersection1.list(),vec![2,4]);
    let bag_intersection2 = bag_b.intersection(&bag_a);
    assert_eq!(bag_intersection2.list(),vec![2,4]);
    let bag_difference1 = bag_a.difference(&bag_b);
    assert_eq!(bag_difference1.list(),vec![1,3]);
    let bag_difference2 = bag_b.difference(&bag_a);
    assert_eq!(bag_difference2.list(),vec![6]);


}
