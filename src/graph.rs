use std::collections::HashMap;

use uuid::Uuid;

use crate::{data::Task, ui::row::{task::TaskRow, RootRow, RowEntry}};

fn get_tasks(tasks: HashMap<Uuid, Task>) -> Vec<RowEntry> {
    let mut task_vec: Vec<_> = tasks
        .iter()
        .map(|(k, t)| (k.clone(), t.sub_of.clone())).collect();
    let mut rows: HashMap<Uuid, RowEntry> = HashMap::new();
    let mut first_loop = true;
    let parents = task_vec.clone();
    let parents: Vec<_> = parents.iter().filter(|(_, p)| p.is_none()).map(|(k, _)| k).collect();
    loop {
        let (level, task_inner): (Vec<_>, Vec<_>)  = task_vec.iter().partition(|(_, p)| {
            match p {
                Some(p_uuid) => {
                    rows.get(p_uuid).is_some()
                }
                None => true,
            }
        });
        task_vec = task_inner;
        if first_loop {
            for (parent, _) in &level {
                rows.insert(parent.clone(), RowEntry::Task(
                        TaskRow {
                            task: tasks[&parent].clone(),
                            sub_tasks: vec![]
                        }
                ));
            }
            first_loop = false;
        } else {
            for (child, parent) in &level {
                match rows.get_mut(&parent.unwrap()) {
                    Some(p) => {
                        p.add_task(RowEntry::Task(
                                TaskRow {
                                    task: tasks[&child].clone(),
                                    sub_tasks: vec![]
                                }
                        ));
                    },
                    None => {
                        rows.insert(child.clone(), RowEntry::Task(
                                TaskRow {
                                    task: tasks[&child].clone(),
                                    sub_tasks: vec![]
                                }
                        ));
                    }
                }
            }
        }
        if task_vec.is_empty() { 
            break; 
        }
    }
    rows.into_iter().filter(|(k, _)| {
        parents.contains(&k)
    }).map(|(_, r)| r).collect()
    // vec![]
}


pub fn into_root(tasks: HashMap<Uuid, Task>) -> RootRow {
    RootRow {
        sub_tasks: get_tasks(tasks)
    }
}
