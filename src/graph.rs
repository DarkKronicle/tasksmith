use std::{cmp::Ordering, collections::{HashMap, VecDeque}};

use strum::IntoEnumIterator;
use uuid::Uuid;

use crate::{data::{Task, TaskStatus}, ui::row::{task::TaskRow, text::TextRow, RootRow, RowEntry}};

pub enum Separation {
    None,
    Status,
}

fn get_tasks(mut tasks: &HashMap<Uuid, Task>, separation: Separation) -> Vec<RowEntry> {
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

    let mut tasks_mut = &mut tasks.clone();

    // Create parent -> child map
    let mut parent_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    for task in tasks.values() {
        parent_map.entry(task.uuid).or_default();
        if let Some(sub_of) = task.sub_of {
            parent_map.entry(sub_of).or_default().push(task.uuid);
        }
    }

    // Tasks that have no parent
    let root_tasks: Vec<_> = tasks.values().filter(|t| t.sub_of.is_none()).map(|t| t.uuid).collect();

    // Depth first stack. We start at the roots.
    let mut task_stack: VecDeque<Uuid> = root_tasks.clone().into();

    // Stack to keep track of the parent TaskRow, so we can modify it easily.
    let mut rows_depth: VecDeque<TaskRow> = VecDeque::new();

    // Rows to return
    let mut rows: Vec<RowEntry> = Vec::new();

    // Start: depth first searc
    while let Some(uuid) = task_stack.pop_back() {
        let task = tasks_mut.remove(&uuid).expect("task went missing");
        let children = parent_map.get(&uuid).expect("built map");
        if !children.is_empty() {
            // Has children, so we create a TaskRow and queue that up.
            let row = TaskRow { task: task.uuid, sub_tasks: vec![] };
            task_stack.extend(children);
            rows_depth.push_back(row);
        } else {
            if rows_depth.is_empty() {
                // We are at root level! So we just add it
                // These are root tasks that have no children
                let row = TaskRow { task: task.uuid, sub_tasks: vec![] };
                rows.push(RowEntry::Task(row));
                continue;
            }
            // We are at a depth != 1. We know this because the depth traversal
            // goes in order.
            
            // Parent uuid
            let par = task.sub_of.expect("task went missing");
            let row = TaskRow { task: task.uuid, sub_tasks: vec![] };
            
            // The parent TaskRow
            let mut one_up = rows_depth.pop_back().expect("depth went missing");
            // add to the parent
            one_up.sub_tasks.push(RowEntry::Task(row));

            // Get the children the parent has, remove our current task.
            let vec = parent_map.get_mut(&par).expect("parent went missing");
            vec.remove(vec.iter().position(|x| *x == uuid).expect("child went missing"));
            if !vec.is_empty() {
                // More children will be traversed, so we add back the parent since
                // next will be another child for this task
                rows_depth.push_back(one_up);
            } else {
                sort_rows(&mut one_up.sub_tasks, tasks);
                // There are no more children, so we have to start building
                // the rows as we traverse upwards.
                
                // NOTE: This is essentially an atomic variable, can probably be changed
                // Length will never be > 1
                let mut ascend = VecDeque::new();
                ascend.push_back(one_up);

                // Make a loop to traverse upwards
                loop {
                    let current = ascend.pop_back().expect("went missing");
                    let task = tasks.get(&current.task).unwrap();
                    if task.sub_of.is_none() {
                        rows.push(RowEntry::Task(current));
                        break;
                    }
                    let par = task.sub_of.expect("task went missing");
                    let vec = parent_map.get_mut(&par).expect("parent went missing");
                    let uuid = task.uuid;
                    vec.remove(vec.iter().position(|x| *x == uuid).expect("child went missing"));

                    let mut one_up = rows_depth.pop_back().expect("went missing");
                    one_up.sub_tasks.push(RowEntry::Task(current));
                    if !vec.is_empty() {
                        rows_depth.push_back(one_up);
                        break;
                    } 
                    ascend.push_back(one_up);
                }
            }

        }
    }
    sort_rows(&mut rows, tasks);
    match separation {
        Separation::None => rows,
        Separation::Status => {
            let mut sorted_statuses = TaskStatus::iter().collect::<Vec<_>>();
            sorted_statuses.sort();
            let mut statuses: HashMap<TaskStatus, Vec<RowEntry>> = TaskStatus::iter().map(|s| (s, vec![])).collect();
            for row in rows.into_iter() {
                if let RowEntry::Task(ref t) = row {
                    let task = tasks.get(&t.task).unwrap();
                    statuses.get_mut(&task.status).unwrap().push(row);
                } else {
                    
                }
            }
            let mut new_rows: Vec<_> = statuses.into_iter().map(|(k, r)| {
                let position = sorted_statuses.iter().position(|s| s == &k).unwrap() as i8;
                RowEntry::Text(TextRow::new(k.to_string(), r, position))
            }).filter(|r| !r.sub_tasks().is_empty()).collect();
            sort_rows(&mut new_rows, tasks);
            new_rows
        }
    }
    // rows
    // vec![]
}


pub fn into_root(tasks: &HashMap<Uuid, Task>, separation: Separation) -> RootRow {
    RootRow {
        sub_tasks: get_tasks(tasks, separation)
    }
}

pub fn sort_rows(rows: &mut [RowEntry], tasks: &HashMap<Uuid, Task>) {
    rows.sort_by(|a, b| {
        if let RowEntry::Task(a) = a {
            if let RowEntry::Task(b) = b {
                let a = tasks.get(&a.task).unwrap();
                let b = tasks.get(&b.task).unwrap();
                if a.status == b.status {
                    let cmp = b.urgency.partial_cmp(&a.urgency).expect("Invalid urgency");
                    return match cmp {
                        Ordering::Equal => {
                            b.description.cmp(&a.description)
                        },
                        _ => cmp
                    }
                }
                return a.status.partial_cmp(&b.status).expect("Invalid status");
            }
        } else if let RowEntry::Text(a) = a {
            if let RowEntry::Text(b) = b {
                return a.sort_by.cmp(&b.sort_by);
            }
        }
        std::cmp::Ordering::Equal
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task_from_row(row: &RowEntry) -> &TaskRow {
        if let RowEntry::Task(t) = row {
            return t
        }
        panic!("Expected task, got: {:?}", row);
    }

    #[test]
    fn root_rows() {
        let t1 = Task::new("t1".to_string());
        let t2 = Task::new("t2".to_string());
        let t3 = Task::new("t3".to_string());
        let t4 = Task::new("t4".to_string());
        let t5 = Task::new("t5".to_string());

        let map: HashMap<_, _> = vec![t1, t2, t3, t4, t5]
            .into_iter()
            .map(|t| (t.uuid, t))
            .collect();

        let rows = get_tasks(&map, Separation::None);
        assert_eq!(rows.len(), 5);
    }

    #[test]
    fn nested_children() {
        let t1 = Task::new("t1".to_string());
        let mut t2 = Task::new("t2".to_string());
        let mut t3 = Task::new("t3".to_string());
        let mut t4 = Task::new("t4".to_string());
        let mut t5 = Task::new("t5".to_string());
        let mut t6 = Task::new("t6".to_string());
        let mut t7 = Task::new("t7".to_string());
        let t8 = Task::new("t8".to_string());
        let t9 = Task::new("t9".to_string());

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

        let rows = get_tasks(&map, Separation::None);
        assert_eq!(rows.len(), 3);
        let s1 = rows.iter().find(|r| task_from_row(r).task == u1).expect("sub went missing");
        assert_eq!(s1.sub_tasks().len(), 2);
        let sub: Vec<_> = s1.sub_tasks().iter().map(|r| task_from_row(r).task).collect();
        assert!(sub.contains(&u2));
        assert!(sub.contains(&u3));

        let s2 = s1.sub_tasks().iter().find(|r| task_from_row(r).task == u2).expect("sub went missing");
        assert_eq!(s2.sub_tasks().len(), 0);

        let s3 = s1.sub_tasks().iter().find(|r| task_from_row(r).task == u3).expect("sub went missing");
        assert_eq!(s3.sub_tasks().len(), 1);

        let s4 = s3.sub_tasks().iter().find(|r| task_from_row(r).task == u4).expect("sub went missing");
        assert_eq!(s4.sub_tasks().len(), 2);

        let s5 = s4.sub_tasks().iter().find(|r| task_from_row(r).task == u5).expect("sub went missing");
        assert_eq!(s5.sub_tasks().len(), 1);

        let s7 = s5.sub_tasks().iter().find(|r| task_from_row(r).task == u7).expect("sub went missing");
        assert_eq!(s7.sub_tasks().len(), 0);

        let s6 = s4.sub_tasks().iter().find(|r| task_from_row(r).task == u6).expect("sub went missing");
        assert_eq!(s6.sub_tasks().len(), 0);
    }
}
