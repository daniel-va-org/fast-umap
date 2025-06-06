use crate::utils::*;
use burn::prelude::*;
use hsl::HSL;
use plotters::{
    prelude::*,
    style::text_anchor::{HPos, Pos, VPos},
};
use std::collections::HashSet;

/// The default caption for the chart
const CAPTION: &str = "fast-umap";

/// The default path where the plot will be saved
const PATH: &str = "plot.png";

/// Configuration structure for the chart, including caption, path, width, and height
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub caption: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
}

impl ChartConfig {
    /// Builder pattern for configuring the chart
    pub fn builder() -> ChartConfigBuilder {
        ChartConfigBuilder {
            caption: Some(CAPTION.to_string()),
            path: Some(PATH.to_string()),
            width: Some(1000),
            height: Some(1000),
        }
    }
}

impl Default for ChartConfig {
    /// Default implementation for ChartConfig with preset values
    fn default() -> Self {
        ChartConfig {
            caption: CAPTION.to_string(),
            path: PATH.to_string(),
            width: 1000,
            height: 1000,
        }
    }
}

/// Builder pattern for `ChartConfig` struct to allow flexible configuration
pub struct ChartConfigBuilder {
    caption: Option<String>,
    path: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

impl Default for ChartConfigBuilder {
    fn default() -> Self {
        ChartConfigBuilder {
            caption: Some(CAPTION.into()),
            path: Some(PATH.into()),
            width: None,
            height: None,
        }
    }
}

impl ChartConfigBuilder {
    /// Set the caption for the chart
    pub fn caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    /// Set the path where the chart will be saved
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    /// Set the width of the chart
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height of the chart
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Build and return the final `ChartConfig`
    pub fn build(self) -> ChartConfig {
        ChartConfig {
            caption: self.caption.unwrap_or_else(|| CAPTION.to_string()),
            path: self.path.unwrap_or_else(|| PATH.to_string()),
            width: self.width.unwrap_or(1000),
            height: self.height.unwrap_or(1000),
        }
    }
}

type Float = f64;

/// Plot the 2D chart using the given tensor data and optional chart configuration
///
/// # Arguments
/// * `data` - A 2D tensor of data points to plot
/// * `config` - Optional custom chart configuration
pub fn chart_tensor<B: Backend>(
    data: Tensor<B, 2>,
    labels: Option<Vec<String>>,
    config: Option<ChartConfig>,
) {
    // pub fn chart_tensor<B: Backend>(data: Tensor<B, 2>, config: Option<ChartConfig>) {
    let data: Vec<Vec<Float>> = convert_tensor_to_vector(data);
    chart_vector(data, labels, config);
}

/// Plot the loss curve over epochs and save it to a file
///
/// # Arguments
/// * `losses` - A vector of loss values over multiple epochs
/// * `output_path` - Path where the plot will be saved
pub fn plot_loss<F: num::Float>(
    losses: Vec<F>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    F:,
{
    // Calculate the min and max loss values
    let min_loss = losses.iter().cloned().fold(F::infinity(), F::min);
    let max_loss = losses.iter().cloned().fold(F::neg_infinity(), F::max);

    // Add padding to the min and max values for better visualization
    let padding = F::from(0.1).unwrap(); // 10% padding, adjust as needed
    let min_loss_with_padding = min_loss - padding * min_loss.abs();
    let max_loss_with_padding = max_loss + padding * max_loss.abs();
    let min_loss_with_padding = min_loss_with_padding.to_f64().unwrap();
    let max_loss_with_padding = max_loss_with_padding.to_f64().unwrap();

    // Create a drawing area with a width of 800px and a height of 600px
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Create a chart builder with padded Y-axis range
    let mut chart = ChartBuilder::on(&root)
        .caption("Loss Over Epochs", ("sans-serif", 30))
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .set_label_area_size(LabelAreaPosition::Bottom, 50)
        .build_cartesian_2d(
            0..losses.len() as u32,
            min_loss_with_padding..max_loss_with_padding,
        )?;

    // Draw the chart axes and grid
    chart
        .configure_mesh()
        .y_desc("Loss")
        .x_desc("Epochs")
        .draw()?;

    // Plot the losses as a line
    chart
        .draw_series(LineSeries::new(
            (0..losses.len()).map(|x| (x as u32, losses[x].to_f64().unwrap())),
            &BLUE,
        ))?
        .label("Loss")
        .legend(move |(x, y)| PathElement::new(vec![(x, y)], &RED));

    // Draw the legend
    chart.configure_series_labels().draw()?;

    // Format Y-axis labels to handle small floats
    chart.configure_mesh().y_labels(10).draw()?;

    Ok(())
}

pub fn chart_vector(
    data: Vec<Vec<Float>>,
    labels: Option<Vec<String>>,
    config: Option<ChartConfig>,
) {
    let config = config.unwrap_or(ChartConfig::default());

    // Create the drawing area
    let root = BitMapBackend::new(&config.path, (config.width, config.height)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    // Calculate min and max for x and y axes
    let min_x = data
        .iter()
        .flat_map(|v| v.iter().step_by(2))
        .cloned()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap() as Float;
    let max_x = data
        .iter()
        .flat_map(|v| v.iter().step_by(2))
        .cloned()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap() as Float;
    let min_y = data
        .iter()
        .flat_map(|v| v.iter().skip(1).step_by(2))
        .cloned()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap() as Float;
    let max_y = data
        .iter()
        .flat_map(|v| v.iter().skip(1).step_by(2))
        .cloned()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap() as Float;

    // Assign colors to unique labels if provided
    let mut label_colors: Vec<(String, RGBColor)> = Vec::new();
    if let Some(labels) = labels.clone() {
        let unique_labels: Vec<String> = labels
            .into_iter()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();
        for (i, label) in unique_labels.iter().enumerate() {
            let hue = i as f64 * 360.0 / unique_labels.len() as f64; // Even distribution of hues
            let color = HSL {
                h: hue,
                s: 0.7,
                l: 0.6,
            }
            .to_rgb();
            label_colors.push((label.clone(), RGBColor(color.0, color.1, color.2)));
        }
    }

    // Build chart
    let mut chart = ChartBuilder::on(&root)
        .caption(config.caption, ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)
        .unwrap();

    // Configure and draw the mesh (axes)
    chart
        .configure_mesh()
        .x_desc("X Axis")
        .y_desc("Y Axis")
        .x_labels(10)
        .y_labels(10)
        .draw()
        .unwrap();

    // Store series for later adding to the legend
    let mut series_list: Vec<(String, Vec<(f64, f64)>, RGBColor)> = Vec::new();

    // Draw data points and labels
    chart
        .draw_series(data.iter().enumerate().map(|(i, values)| {
            let label = labels
                .clone()
                .map(|l| l.get(i).cloned())
                .flatten()
                .unwrap_or_else(|| "".into());
            let color = label_colors
                .iter()
                .find(|(l, _)| *l == label)
                .map(|(_, c)| *c)
                .unwrap_or(RED);

            // Store series data for the legend
            if !label.is_empty() {
                let series_data = series_list.iter_mut().find(|(l, _, _)| *l == label);
                match series_data {
                    Some((_, series_points, _)) => series_points.push((values[0], values[1])),
                    None => series_list.push((label.clone(), vec![(values[0], values[1])], color)),
                }
            }

            // Draw circle for each point
            Circle::new(
                (values[0], values[1]),
                3,
                ShapeStyle {
                    color: color.into(),
                    filled: false,
                    stroke_width: 1,
                },
            )
        }))
        .unwrap();

    // Add the legend manually
    if labels.is_some() {
        // Sort the series list alphabetically by label
        series_list.sort_by(|a, b| {
            let a = a.0.parse::<usize>().unwrap();
            let b = b.0.parse::<usize>().unwrap();
            a.cmp(&b)
            // a.0.cmp(&b.0)
        });

        let spacing_y = (max_y - min_y) / (series_list.len() * 2) as f64;

        // Define the starting position for the legend
        let mut legend_position = (min_x + (max_x - min_x) * 0.8, max_y - (max_y - min_y) * 0.1);
        // let spacing = 10.0; // Increase the spacing
        let size = 5.0; // Make the circles slightly larger
        let font_size = 15.0;

        for (label, _, color) in series_list {
            // Draw a colored circle for each label in the legend
            chart
                .draw_series(std::iter::once(Circle::new(
                    legend_position,
                    size,
                    ShapeStyle {
                        color: color.into(),
                        filled: true,
                        stroke_width: 1,
                    },
                )))
                .unwrap();

            let style = TextStyle {
                font: ("sans-serif", font_size).into_font(),
                color: BLACK.to_backend_color(),
                pos: Pos::new(HPos::Left, VPos::Center),
            };

            // Draw the label text next to the circle
            chart
                .draw_series(std::iter::once(Text::new(
                    label,
                    (legend_position.0 + spacing_y / 4.0, legend_position.1),
                    style,
                )))
                .unwrap();

            // Move the position for the next legend item downwards
            legend_position.1 -= spacing_y;
        }
    }

    // Save the chart to file
    root.present().unwrap();
}
