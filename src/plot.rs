use super::profile::Profile;
use plotters::prelude::*;
use std::cmp::max;

pub fn plot(prof: &mut Profile) {
    let root_area = SVGBackend::new("plot.svg", (1000, 450)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let max_x = prof.elapsed;
    let max_y = max(prof.virtual_peak, prof.real_peak) as f32 * 0.00110;

    let mut ctx = ChartBuilder::on(&root_area)
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .set_label_area_size(LabelAreaPosition::Right, 80)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .caption("Memory profile", ("Ubuntu", 40).into_font())
        .build_cartesian_2d(0f32..max_x, 0f32..max_y)
        .unwrap();

    ctx.configure_mesh().draw().unwrap();

    let data = prof.records().unwrap();

    ctx.draw_series(LineSeries::new(
        data.iter().map(|x| (x.0, x.1 as f32 * 0.001)),
        &RED,
    ))
    .unwrap();
    ctx.draw_series(LineSeries::new(
        data.iter().map(|x| (x.0, x.2 as f32 * 0.001)),
        &GREEN,
    ))
    .unwrap();
    ctx.draw_series(LineSeries::new(
        data.iter().map(|x| (x.0, x.3 as f32 * 0.001)),
        &BLUE,
    ))
    .unwrap();
    ctx.draw_series(LineSeries::new(
        data.iter().map(|x| (x.0, x.4 as f32 * 0.001)),
        &YELLOW,
    ))
    .unwrap();
}
