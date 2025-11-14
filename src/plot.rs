use crate::Float;
use plotters::prelude::*;

pub fn plot(data: &Vec<Float>, t_final: Float, dt: Float, n_steps: usize, fname: &str) {
    // Determine y-axis limits based on the minimum and maximum values in the data
    let min_y = data.iter().cloned().fold(Float::INFINITY, Float::min);
    let max_y = data.iter().cloned().fold(Float::NEG_INFINITY, Float::max);

    // Create a plotting area
    let file_name = format!("{}.png", fname);
    let root = BitMapBackend::new(&file_name, (640, 480)).into_drawing_area();
    let _ = root.fill(&WHITE);

    // Configure the chart
    let mut chart = ChartBuilder::on(&root)
        .caption("x vs. Time", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(0.0..t_final, min_y..max_y)
        .unwrap();

    // Customize the chart
    let _ = chart.configure_mesh().draw();

    // Plot the data
    let _ = chart.draw_series(LineSeries::new(
        (0..n_steps).map(|i| (i as Float * dt, data[i])),
        &BLUE,
    ));
}
