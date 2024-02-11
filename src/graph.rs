use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

use crate::{data::Task, ui::row::{task::TaskRow, RootRow, RowEntry}};

fn get_tasks(mut tasks: HashMap<Uuid, Task>) -> Vec<RowEntry> {
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



    // Create parent -> child map
    let mut parent_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();

    for task in tasks.values() {
        parent_map.entry(task.uuid.clone()).or_insert_with(Vec::new);
        if let Some(sub_of) = task.sub_of {
            parent_map.entry(sub_of.clone()).or_insert_with(Vec::new).push(task.uuid.clone());
        }
    }

    // Tasks that have no parent
    let root_tasks: Vec<_> = tasks.values().filter(|t| t.sub_of.is_none()).map(|t| t.uuid.clone()).collect();

    // Depth first stack. We start at the roots.
    let mut task_stack: VecDeque<Uuid> = root_tasks.clone().into();

    // Stack to keep track of the parent TaskRow, so we can modify it easily.
    let mut rows_depth: VecDeque<TaskRow> = VecDeque::new();

    // Rows to return
    let mut rows: Vec<RowEntry> = Vec::new();

    // Start: depth first searc
    while let Some(uuid) = task_stack.pop_back() {
        let task = tasks.remove(&uuid).expect("task went missing");
        let children = parent_map.get(&uuid).expect("built map");
        if !children.is_empty() {
            // Has children, so we create a TaskRow and queue that up.
            let row = TaskRow { task, sub_tasks: vec![] };
            task_stack.extend(children);
            rows_depth.push_back(row);
        } else {
            if rows_depth.is_empty() {
                // We are at root level! So we just add it
                // These are root tasks that have no children
                let row = TaskRow { task, sub_tasks: vec![] };
                rows.push(RowEntry::Task(row));
                continue;
            }
            // We are at a depth != 1. We know this because the depth traversal
            // goes in order.
            
            // Parent uuid
            let par = task.sub_of.expect("task went missing").clone();
            let row = TaskRow { task, sub_tasks: vec![] };
            
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
                // There are no more children, so we have to start building
                // the rows as we traverse upwards.
                
                // This is essentially an atomic variable, can probably be changed
                // Length will never be > 1
                let mut ascend = VecDeque::new();
                ascend.push_back(one_up);

                // Make a loop to traverse upwards
                loop {
                    let current = ascend.pop_back().expect("went missing");
                    if current.task.sub_of.is_none() {
                        rows.push(RowEntry::Task(current));
                        break;
                    }
                    let par = current.task.sub_of.expect("task went missing").clone();
                    let vec = parent_map.get_mut(&par).expect("parent went missing");
                    let uuid = current.task.uuid;
                    vec.remove(vec.iter().position(|x| *x == uuid).expect("child went missing"));

                    let mut one_up = rows_depth.pop_back().expect("went missing");
                    one_up.sub_tasks.push(RowEntry::Task(current));
                    // if rows_depth.is_empty() {
                    //     rows.push(RowEntry::Task(one_up));
                    //     break;
                    // } else 
                    if !vec.is_empty() {
                        rows_depth.push_back(one_up);
                        break;
                    } 
                    // if rows_depth.is_empty() {
                    //     rows.push(RowEntry::Task(one_up));
                    //     break;
                    // }
                    ascend.push_back(one_up);
                }
            }

        }
    }
    rows
    // vec![]
}


pub fn into_root(tasks: HashMap<Uuid, Task>) -> RootRow {
    RootRow {
        sub_tasks: get_tasks(tasks)
    }
}

pub fn sort_rows(rows: &mut [RowEntry]) {
    rows.sort_by(|a, b| {
        if let RowEntry::Task(a) = a {
            if let RowEntry::Task(b) = b {
                if a.task.status == b.task.status {
                    return b.task.urgency.partial_cmp(&a.task.urgency).expect("Invalid urgency");
                }
                return a.task.status.partial_cmp(&b.task.status).expect("Invalid status");
            }
        } else if let RowEntry::Text(a) = a {
            if let RowEntry::Text(b) = b {
                return a.sort_by.cmp(&b.sort_by);
            }
        }
        std::cmp::Ordering::Equal
    });
}
