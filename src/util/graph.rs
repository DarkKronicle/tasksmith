use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Node {

    val: Uuid,
    sub: Vec<Node>
    
}

impl Node {

    pub fn child_len(&self) -> usize {
        self.sub.len()
    }

    pub fn contains(&self, uuid: &Uuid) -> bool {
        self.sub.iter().any(|s| s.get_id() == *uuid)
    }

    pub fn try_get(&self, uuid: &Uuid) -> Option<&Node> {
        self.sub.iter().find(|s| s.get_id() == *uuid)
    }



}

impl Idable for Node {
    fn get_id(&self) -> Uuid {
        self.val
    }

    fn get_id_ref(&self) -> &Uuid {
        &self.val
    }

}

pub trait Idable {

    fn get_id(&self) -> Uuid;
    fn get_id_ref(&self) -> &Uuid;

}

pub trait ParentToChild {

    fn sub_of(&self) -> Option<Uuid>;
    fn get_id(&self) -> Uuid;
    fn get_id_ref(& self) -> &Uuid;

}



fn graph_nodes<T: ParentToChild>(id_map: &HashMap<Uuid, T>) -> Vec<Node> {
    // Ok, so this is a doozey of an algorithm. I'll explain it here:
    //
    // First of all, why this implementation?
    //   - No recursion. This prevents having to clone as we go deeper and deeper
    //   - We only clone uuids which are relatively cheap (at least compared to rows and tasks)
    //   - We have access to the parent at any time
    //   - Fairly easy to expand if wanted to add sections or some other type here
    // 
    // We first start with making a map of parent -> children. 
    //   This is so we can easily tell if we are done with a row entry or not
    //
    // We then create a list of "root" tasks. These are tasks that are sub of no one.
    //   This sets the stage for a depth first traversal. We start at the top tasks
    //   and continually go downwards.
    //
    // There are 3 variables here:
    //   - task_stack: the traversal stack
    //   - rows_depth: a stack of the current depth
    //   -- These are essentially TaskRow's that have been
    //      created but we aren't entirely sure if we're going to
    //      need them again. This allows us to modify them 
    //      and then add them to the stack.
    //   - rows: the rows that are all done and ready to be sent
    //
    // During depth first search we check for 2 conditions:
    //   Current task has children:
    //   - Add children to task_stack 
    //   - Create a row, and add that to rows_depth
    //
    //   Current task has no children:
    //    We are now at the "bottom" of a branch. If we're at the top (rows_depth)
    //    is empty, we can just add to rows and be done, if not, we need to traverse upwards.
    //
    //    The traverse goes up one layer, adds the task to that row, removes itself
    //    as a child from the parent (the parent -> child map). This serves as a counter
    //    of how many children still need to be processed. If there are more children,
    //    we re-add that parent and then continue.
    //
    //    If there are no more children, we continue to go up, repeating the same process,
    //    until there are either more children again, or we're at root level.
    //
    // Current problems:
    // - Does not handle closed loops... it will probably panic
    // - Code uses way too many expects
    // - Could probably de-duplicate some code (the loop mainly)

    let mut id_mut: HashMap<Uuid, Uuid> = id_map.iter().map(|(u, t)| (*u, t.get_id())).collect();

    // Create parent -> child map
    let mut parent_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    for val in id_map.values() {
        parent_map.entry(val.get_id()).or_default();
        if let Some(sub_of) = val.sub_of() {
            parent_map.entry(sub_of).or_default().push(val.get_id());
        }
    }

    // Values that have no parent
    let root_vals: Vec<_> = id_map.values().filter(|t| t.sub_of().is_none()).map(|t| t.get_id()).collect();

    // Depth first stack. We start at the roots.
    let mut task_stack: VecDeque<Uuid> = root_vals.clone().into();

    // Stack to keep track of the parent TaskRow, so we can modify it easily.
    let mut rows_depth: VecDeque<Node> = VecDeque::new();

    // Rows to return
    let mut rows: Vec<Node> = Vec::new();

    // Start: depth first searc
    while let Some(uuid) = task_stack.pop_back() {
        
        let val_uuid = id_mut.remove(&uuid).expect("task went missing");
        let val = id_map.get(&uuid).expect("task went missing");
        let children = parent_map.get(&uuid).expect("built map");
        if !children.is_empty() {
            // Has children, so we create a TaskRow and queue that up.
            let row = Node { val: val_uuid, sub: vec![] };
            task_stack.extend(children);
            rows_depth.push_back(row);
        } else {
            if rows_depth.is_empty() {
                // We are at root level! So we just add it
                // These are root tasks that have no children
                let row = Node { val: val_uuid, sub: vec![] };
                rows.push(row);
                continue;
            }
            // We are at a depth != 1. We know this because the depth traversal
            // goes in order.
            
            // Parent uuid
            let par = val.sub_of().expect("task went missing");
            let row = Node { val: val_uuid, sub: vec![] };
            
            // The parent TaskRow
            let mut one_up = rows_depth.pop_back().expect("depth went missing");
            // add to the parent
            one_up.sub.push(row);

            // Get the children the parent has, remove our current task.
            let vec = parent_map.get_mut(&par).expect("parent went missing");
            vec.remove(vec.iter().position(|x| *x == uuid).expect("child went missing"));
            if !vec.is_empty() {
                // More children will be traversed, so we add back the parent since
                // next will be another child for this task
                rows_depth.push_back(one_up);
            } else {
                // There are no more children, so we have to start building
                // the rows as we traverse upwards.
                
                // NOTE: This is essentially an atomic variable, can probably be changed
                // Length will never be > 1
                let mut ascend = VecDeque::new();
                ascend.push_back(one_up);

                // Make a loop to traverse upwards
                loop {
                    let current = ascend.pop_back().expect("went missing");
                    let task = id_map.get(current.get_id_ref()).unwrap();
                    if task.sub_of().is_none() {
                        rows.push(current);
                        break;
                    }
                    let par = task.sub_of().expect("task went missing");
                    let vec = parent_map.get_mut(&par).expect("parent went missing");
                    let uuid = task.get_id();
                    vec.remove(vec.iter().position(|x| *x == uuid).expect("child went missing"));

                    let mut one_up = rows_depth.pop_back().expect("went missing");
                    one_up.sub.push(current);
                    if !vec.is_empty() {
                        rows_depth.push_back(one_up);
                        break;
                    } 
                    ascend.push_back(one_up);
                }
            }

        }
    }
    rows
}



#[cfg(test)]
mod tests {
    use super::*;

    struct TestParent {
        uuid: Uuid,
        sub_of: Option<Uuid>,
    }

    impl ParentToChild for TestParent {
        fn get_id(&self) -> Uuid {
            self.uuid.clone()
        }

        fn get_id_ref<'a>(&'a self) -> &'a Uuid {
            &self.uuid
        }


        fn sub_of(&self) -> Option<Uuid> {
            self.sub_of.clone()
        }
    }

    impl TestParent {
        fn new() -> TestParent {
            TestParent { uuid: Uuid::new_v4(), sub_of: None }
        }
    }

    struct TestNode {
        uuid: Uuid,
    }

    impl Idable for TestNode {
        fn get_id(&self) -> Uuid {
            self.uuid.clone()
        }

        fn get_id_ref<'a>(&'a self) -> &'a Uuid {
            &self.uuid
        }

    }


    #[test]
    fn root_rows() {
        let t1 = TestParent::new();
        let t2 = TestParent::new();
        let t3 = TestParent::new();
        let t4 = TestParent::new();
        let t5 = TestParent::new();

        let map: HashMap<_, _> = vec![t1, t2, t3, t4, t5]
            .into_iter()
            .map(|t| (t.uuid, t))
            .collect();

        let rows = graph_nodes(&map);
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn nested_children() {
        let mut t1 = TestParent::new();
        let mut t2 = TestParent::new();
        let mut t3 = TestParent::new();
        let mut t4 = TestParent::new();
        let mut t5 = TestParent::new();
        let mut t6 = TestParent::new();
        let mut t7 = TestParent::new();
        let t8 = TestParent::new();
        let t9 = TestParent::new();

        let u1 = t1.uuid;
        let u2 = t2.uuid;
        let u3 = t3.uuid;
        let u4 = t4.uuid;
        let u5 = t5.uuid;
        let u6 = t6.uuid;
        let u7 = t7.uuid;
        // let u8 = t8.uuid;
        // let u9 = t9.uuid;

        t7.sub_of = Some(u5);
        t6.sub_of = Some(u4);
        t5.sub_of = Some(u4);
        t4.sub_of = Some(u3);
        t3.sub_of = Some(u1);
        t2.sub_of = Some(u1);

        let map: HashMap<_, _> = vec![t1, t2, t3, t4, t5, t6, t7, t8, t9]
            .into_iter()
            .map(|t| (t.uuid, t))
            .collect();

        let rows = graph_nodes(&map);
        assert_eq!(rows.len(), 3);
        let s1 = rows.iter().find(|r| r.get_id() == u1).expect("sub went missing");
        assert_eq!(s1.child_len(), 2);
        assert!(s1.contains(&u2));
        assert!(s1.contains(&u3));

        let s2 = s1.try_get(&u2).expect("sub went missing");
        assert_eq!(s2.child_len(), 0);
        let s3 = s1.try_get(&u3).expect("sub went missing");
        assert_eq!(s3.child_len(), 1);

        let s4 = s3.try_get(&u4).expect("sub went missing");
        assert_eq!(s4.child_len(), 2);
        let s5 = s4.try_get(&u5).expect("sub went missing");
        assert_eq!(s5.child_len(), 1);
        let s7 = s5.try_get(&u7).expect("sub went missing");

        assert_eq!(s7.child_len(), 0);

        let s6 = s4.try_get(&u6).expect("sub went missing");
        assert_eq!(s6.child_len(), 0);
    }
}

