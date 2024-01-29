use color_eyre::eyre::Result;

mod display;
mod task;
mod ui;


fn main() -> Result<()> {
    color_eyre::install()?;
    let mut tasks = task::get_tasks(Some("+PENDING"))?;
    tasks.tasks.sort_unstable_by(|a, b| {
        let a = a.urgency;
        let b = b.urgency;
        a.partial_cmp(&b).unwrap().reverse()
    });
    display::display_table(tasks)?;
    Ok(())
}
