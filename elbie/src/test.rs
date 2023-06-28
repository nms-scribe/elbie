
#[test]
fn test_bags() {
    use super::Bag;

    let mut bag_a = Bag::new();
    bag_a.insert(1);
    bag_a.insert(2);
    bag_a.insert(3);
    bag_a.insert(4);

    let mut bag_b = Bag::new();
    bag_b.insert(2);
    bag_b.insert(4);
    bag_b.insert(6);

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