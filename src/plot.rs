use super::profile::Profile;
use gnuplot::{AutoOption, AxesCommon, Caption, Figure};

pub fn plot(prof: &mut Profile) {
    let data = prof.records().unwrap();

    let x: Vec<f32> = data.iter().map(|r| r.0).collect();
    let r_size: Vec<f32> = data.iter().map(|r| r.1 as f32 / 1024f32).collect();
    let r_peak: Vec<f32> = data.iter().map(|r| r.2 as f32 / 1024f32).collect();
    let v_size: Vec<f32> = data.iter().map(|r| r.3 as f32 / 1024f32).collect();
    let v_peak: Vec<f32> = data.iter().map(|r| r.4 as f32 / 1024f32).collect();

    let mut fg = Figure::new();

    fg.axes2d()
        .set_x_grid(true)
        .set_x_minor_grid(true)
        .set_x_ticks(Some((AutoOption::Auto, 1)), &[], &[])
        .set_x_label("Time (s)", &[])
        .set_y_grid(true)
        .set_y_ticks(Some((AutoOption::Auto, 1)), &[], &[])
        .set_y_minor_grid(true)
        .set_y_label("Footprint (MB)", &[])
        .lines(&x, &r_size, &[Caption("Real Memory")])
        .lines(&x, &r_peak, &[Caption("Peak Real Memory")])
        .lines(&x, &v_size, &[Caption("Virtual memory size")])
        .lines(&x, &v_peak, &[Caption("Peak virtual memory size")]);

    if let Err(e) = fg.show() {
        eprintln!("{}", e);
    }
}
