use self::{
    bounder::Bounded,
    config::{Mesh, Svg},
    normalizer::Normalized,
};
use crate::{
    parser::Parsed,
    utils::{with_index, BoundExt, Display, DroppedFileExt, RangeBoundsExt, UiExt},
};
use anyhow::{Error, Result};
use eframe::{
    epaint::Hsva,
    get_value,
    glow::{HasContext, PixelPackData, RGB, RGBA, UNSIGNED_BYTE},
    set_value, CreationContext, Frame, Storage, APP_KEY,
};
use egui::{
    global_dark_light_mode_switch,
    menu::bar,
    plot::{
        self, log_grid_spacer, uniform_grid_spacer, Bar, BarChart, CoordinatesFormatter, Corner,
        HLine, Legend, Line as DLine, MarkerShape, Plot, PlotBounds, PlotPoint, PlotPoints, Points,
        Text, VLine,
    },
    text::LayoutJob,
    warn_if_debug_build, Align, Align2, CentralPanel, Color32, ColorImage, Context, DragValue,
    DroppedFile, FontData, FontDefinitions, FontFamily, FontId, Id, LayerId, Layout, Order,
    Response, RichText, SidePanel, Slider, TextEdit, TextStyle, TopBottomPanel, Ui, Vec2,
    WidgetText, Window,
};
use egui_extras::RetainedImage;
use image::{imageops::*, ColorType, DynamicImage, GrayImage, ImageResult, RgbaImage};
use indexmap::{indexmap, IndexMap};
use ndarray::{Array1, Dimension};
use ndarray_stats::{interpolate::Linear, Quantile1dExt};
use noisy_float::types::n64;
use plotters::{prelude::*, style::RelativeSize};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    default::default,
    fmt::{self, Write},
    ops::{Bound, Deref, DerefMut, Range, RangeBounds, RangeInclusive},
    path::{Path, PathBuf},
};
use tracing::{error, info};

const COLOR: Color32 = Color32::BLACK;

macro font($fonts: ident, $name: literal) {
    $fonts.font_data.insert(
        $name.to_owned(),
        FontData::from_static(include_bytes!(concat!("../../fonts/", $name, ".ttf"))),
    );
    $fonts.font_data.insert(
        concat!($name, " Bold").to_owned(),
        FontData::from_static(include_bytes!(concat!("../../fonts/", $name, "_Bold.ttf"))),
    );
    $fonts.font_data.insert(
        concat!($name, " Italic").to_owned(),
        FontData::from_static(include_bytes!(concat!(
            "../../fonts/",
            $name,
            "_Italic.ttf"
        ))),
    );
    $fonts.font_data.insert(
        concat!($name, " Bold Italic").to_owned(),
        FontData::from_static(include_bytes!(concat!(
            "../../fonts/",
            $name,
            "_Bold_Italic.ttf"
        ))),
    );
    $fonts
        .families
        .entry(FontFamily::Name($name.into()))
        .or_default()
        .insert(0, $name.to_owned());
    $fonts.families.insert(
        FontFamily::Name(concat!($name, " Bold").into()),
        vec![concat!($name, " Bold").to_owned()],
    );
    $fonts.families.insert(
        FontFamily::Name(concat!($name, " Italic").into()),
        vec![concat!($name, " Italic").to_owned()],
    );
    $fonts.families.insert(
        FontFamily::Name(concat!($name, " Bold Italic").into()),
        vec![concat!($name, " Bold Italic").to_owned()],
    );
}

pub fn color(index: usize) -> Color32 {
    let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
    let h = index as f32 * golden_ratio;
    Hsva::new(h, 0.85, 0.5, 1.0).into()
}

fn save_image(image: &ColorImage, path: &Path) -> ImageResult<()> {
    let height = image.height();
    let width = image.width();
    let mut buf: Vec<u8> = vec![];
    for color in &image.pixels {
        buf.push(color.r() & color.g() & color.b())
    }
    let luma8 = GrayImage::from_raw(width as _, height as _, buf)
        .expect("container should have the right size for the image dimensions");
    luma8.save(path)
    // for color in &image.pixels {
    //     buf.push(color.r());
    //     buf.push(color.g());
    //     buf.push(color.b());
    //     buf.push(color.a());
    // }
    // let rgba8 = RgbaImage::from_raw(width as _, height as _, buf)
    //     .expect("container should have the right size for the image dimensions");
    // let mut luma8 = DynamicImage::ImageRgba8(rgba8).grayscale().to_luma8();
    // // dither(&mut luma8, &BiLevel);
    // luma8.save(path)
}

fn setup_fonts(ctx: &Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = FontDefinitions::default();
    font!(fonts, "Arial");
    // Put the Arial font first (highest priority) for proportional text.
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Arial".to_owned());

    // Put the Arial font as last fallback for monospace.
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("Arial".to_owned());
    // Tell egui to use these fonts.
    ctx.set_fonts(fonts);
}

mod config {
    use serde::{Deserialize, Serialize};

    /// Mesh
    #[derive(Deserialize, Serialize)]
    pub(super) struct Mesh {
        pub(super) x_desc: String,
        pub(super) y_desc: String,
    }

    impl Default for Mesh {
        fn default() -> Self {
            Self {
                x_desc: "m/z".to_string(),
                y_desc: "Blabalbla %".to_string(),
            }
        }
    }

    // Size
    #[derive(Clone, Copy, Deserialize, Serialize)]
    pub(super) struct Svg {
        pub(super) size: (u32, u32),
    }

    impl Default for Svg {
        fn default() -> Self {
            Self { size: (1920, 1080) }
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    files: Vec<DroppedFile>,
    parsed: HashMap<usize, Parsed>,
    colors: IndexMap<usize, Color32>,
    filter: HashSet<usize>,

    left_panel: bool,

    // Filter
    bounds: Bounds,

    // Visual
    // font: &'static str,
    svg: Svg,
    test: u32,
    x_label_area_size: f64,
    y_label_area_size: f64,
    margin: f64,
    tick_mark_size: f64,
    stroke_width: u32,
    mesh: Mesh,

    margin1: u32,

    visuals: Visuals,
    labels: Vec<Label>,
    points: Vec<Point>,

    window_size: Vec2,
    plot_size: [f32; 4],
    save_plot: bool,
    plot_to_save: Option<ColorImage>,

    #[serde(skip)]
    errors: Errors,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
        // Customize style of egui.
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.collapsing_header_frame = true;
        cc.egui_ctx.set_style(style);
        setup_fonts(&cc.egui_ctx);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }

    fn drag_and_drop_files(&mut self, ctx: &Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(files) = ctx.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?files);
            self.files = files;
            for (index, file) in self.files.iter().enumerate() {
                let content = match file.content() {
                    Ok(content) => content,
                    Err(error) => {
                        error!(%error);
                        self.errors.buffer.insert(index, error);
                        continue;
                    }
                };
                let parsed = match content.parse() {
                    Ok(file) => file,
                    Err(error) => {
                        error!(%error);
                        self.errors.buffer.insert(index, error);
                        continue;
                    }
                };
                self.parsed.insert(index, parsed);
                self.colors.insert(index, color(index));
            }
        }
    }

    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    warn_if_debug_build(ui);
                    ui.spacing();
                    ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                });
            });
        });
    }

    fn central_panel(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.files.is_empty() {
                ui.centered_and_justified(|ui| ui.label("Drag and drop .msp file"));
            } else {
                ui.vertical_centered_justified(|ui| {
                    ui.heading(&self.parsed[&0].name);
                });
                ui.separator();
                let desired_size = ui.available_size();
                let svg = self.svg(ui).unwrap();
                RetainedImage::from_svg_bytes("chart", &svg)
                    .unwrap()
                    .show_size(ui, desired_size);
            }
        });
        // Save plot
        if let Some(image) = self.plot_to_save.take() {
            let path = Path::new("temp_plot.png");
            match save_image(&image, &path) {
                Ok(_) => {
                    println!("saving ok!")
                }
                Err(error) => {
                    println!("failed to plot to {:?}: {:?}", path, error);
                }
            }
        }
    }

    fn left_panel(&mut self, ctx: &Context) {
        SidePanel::left("left_panel").show_animated(ctx, self.left_panel, |ui| {
            ui.heading("Left Panel");
            ui.separator();
            // Chart
            ui.collapsing(WidgetText::from("Chart").heading(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(DragValue::new(&mut self.svg.size.0).speed(1))
                        .on_hover_text("X");
                    ui.add(DragValue::new(&mut self.svg.size.1).speed(1))
                        .on_hover_text("Y");
                });
                ui.label("Mesh:");
                ui.group(|ui| {
                    ui.label("Descriptions:");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.text_edit_singleline(&mut self.mesh.x_desc);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        ui.text_edit_singleline(&mut self.mesh.y_desc);
                    });
                });
            });
            // Axes
            ui.collapsing(WidgetText::from("Axes").heading(), |ui| {
                ui.group(|ui| {
                    ui.label("Style:");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Stroke width:");
                        ui.add(
                            DragValue::new(&mut self.stroke_width)
                                .clamp_range(0..=u32::MAX)
                                .speed(1),
                        );
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Tick mark size:");
                    ui.add(
                        DragValue::new(&mut self.tick_mark_size)
                            .clamp_range(0.0..=1.0)
                            .speed(0.0001),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Label area size:");
                    ui.add(DragValue::new(&mut self.x_label_area_size).speed(1))
                        .on_hover_text("X");
                    ui.add(DragValue::new(&mut self.y_label_area_size).speed(1))
                        .on_hover_text("Y");
                });
                ui.horizontal(|ui| {
                    ui.label("Margin:");
                    ui.add(
                        DragValue::new(&mut self.margin)
                            .clamp_range(0.0..=f64::MAX)
                            .speed(1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Test:");
                    ui.add(DragValue::new(&mut self.test).speed(1));
                });
                ui.horizontal(|ui| {
                    ui.label("Margin:");
                    ui.add(
                        DragValue::new(&mut self.margin1)
                            .clamp_range(0..=u32::MAX)
                            .speed(1),
                    );
                });
            });
            // Bounds
            ui.collapsing(WidgetText::from("Bounds").heading(), |ui| {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Mass:");
                    ui.add(
                        DragValue::new(&mut self.bounds.x.start)
                            .clamp_range(0..=self.bounds.x.end)
                            .speed(1),
                    );
                    ui.add(
                        DragValue::new(&mut self.bounds.x.end)
                            .clamp_range(self.bounds.x.start..=u64::MAX)
                            .speed(1),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(
                        DragValue::new(&mut self.bounds.y.start)
                            .clamp_range(0..=self.bounds.y.end)
                            .speed(1),
                    );
                    ui.add(
                        DragValue::new(&mut self.bounds.y.end)
                            .clamp_range(self.bounds.y.start..=100)
                            .speed(1),
                    );
                });
            });
            ui.collapsing(WidgetText::from("Visual").heading(), |ui| {
                ui.separator();
                ui.heading("Plot");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(
                        DragValue::new(&mut self.visuals.width)
                            .clamp_range(0.0..=f32::MAX)
                            .speed(1),
                    );
                });
                ui.separator();
                ui.heading("Chart");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.add(
                        DragValue::new(&mut self.visuals.chart.width)
                            .clamp_range(0.0..=f64::MAX)
                            .speed(0.01),
                    );
                });
                ui.separator();
                ui.heading("Axes");
                ui.separator();
                ui.group(|ui| {
                    ui.label("Unlabeled:");
                    ui.horizontal(|ui| {
                        ui.label("Step:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.unlabeled.step)
                                .clamp_range(1..=usize::MAX)
                                .speed(1),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.unlabeled.height)
                                .clamp_range(0.0..=f64::MAX)
                                .speed(1.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.unlabeled.width)
                                .clamp_range(0.0..=f64::MAX)
                                .speed(1.0),
                        );
                    });
                });
                ui.group(|ui| {
                    ui.label("Labeled:");
                    ui.horizontal(|ui| {
                        ui.label("Step:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.labeled.step)
                                .clamp_range(1..=usize::MAX)
                                .speed(1),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.labeled.height)
                                .clamp_range(0.0..=f64::MAX)
                                .speed(1.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.labeled.width)
                                .clamp_range(0.0..=f64::MAX)
                                .speed(1.0),
                        );
                    });
                    ui.label("Font:");
                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        ui.add(
                            DragValue::new(&mut self.visuals.x.labeled.font_size)
                                .clamp_range(1.0..=f64::MAX)
                                .speed(1.0),
                        );
                    })
                    .response
                    .on_hover_text("Height in points");
                });
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Step:");
                        ui.add(
                            DragValue::new(&mut self.visuals.y.step)
                                .clamp_range(1..=usize::MAX)
                                .speed(1),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(
                            DragValue::new(&mut self.visuals.y.height)
                                .clamp_range(0.0..=f64::MAX)
                                .speed(1.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(
                            DragValue::new(&mut self.visuals.y.width)
                                .clamp_range(0.0..=f64::MAX)
                                .speed(1.0),
                        );
                    });
                });
                // ui.color_edit_button_srgba(&mut self.visuals.division.color);
            });
            // Labels
            ui.collapsing(WidgetText::from("Labels").heading(), |ui| {
                self.labels.retain_mut(|label| {
                    ui.horizontal(|ui| {
                        ui.label("Label:");
                        let width = 4.0 * ui.text_style_height(&TextStyle::Body);
                        TextEdit::singleline(&mut label.text)
                            .desired_width(width)
                            .show(ui);
                        ui.add(DragValue::new(&mut label.coordinates.x).speed(1))
                            .on_hover_text("X")
                            .context_menu(|ui| {
                                if ui.button("Center").clicked() {
                                    label.coordinates.x = self.bounds.x.center();
                                    ui.close_menu();
                                }
                            });
                        ui.add(DragValue::new(&mut label.coordinates.y).speed(1))
                            .on_hover_text("Y")
                            .context_menu(|ui| {
                                if ui.button("Center").clicked() {
                                    // label.coordinates.y = self.bounds.y.center();
                                    ui.close_menu();
                                }
                            });
                        ui.toggle_value(&mut label.bold, "bold");
                        !ui.button(RichText::new("-").monospace()).clicked()
                    })
                    .inner
                });
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("-").monospace()).clicked() {
                        self.labels.clear();
                    }
                    if ui.button(RichText::new("+").monospace()).clicked() {
                        self.labels.push(default());
                    }
                });
            });
            // Points
            ui.collapsing(WidgetText::from("Points").heading(), |ui| {
                self.points.retain_mut(|point| {
                    ui.horizontal(|ui| {
                        ui.label("Point:");
                        ui.add(DragValue::new(&mut point.coordinates.x).speed(1))
                            .on_hover_text("X")
                            .context_menu(|ui| {
                                if ui.button("Center").clicked() {
                                    point.coordinates.x = self.bounds.x.center();
                                    ui.close_menu();
                                }
                            });
                        ui.add(DragValue::new(&mut point.coordinates.y).speed(1))
                            .on_hover_text("Y")
                            .context_menu(|ui| {
                                if ui.button("Center").clicked() {
                                    // point.coordinates.y = self.bounds.y.center();
                                    ui.close_menu();
                                }
                            });
                        ui.toggle_value(&mut point.filled, "filled");
                        ui.add(DragValue::new(&mut point.size).speed(1))
                            .on_hover_text("Radius");
                        ui.color_edit_button_srgba(&mut point.color)
                            .on_hover_text("Color");
                        !ui.button(RichText::new("-").monospace()).clicked()
                    })
                    .inner
                });
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("+").monospace()).clicked() {
                        self.points.push(default());
                    }
                });
            });
        });
    }

    fn top_panel(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                global_dark_light_mode_switch(ui);
                ui.separator();
                ui.toggle_value(&mut self.left_panel, "🛠 Control");
                ui.toggle_value(&mut self.errors.show, "⚠ Errors");
                if ui.button("Save Plot").clicked() {
                    self.save_plot = true;
                    std::fs::write("output.svg", self.svg(ui).unwrap()).unwrap();
                }
            });
        });
    }

    fn errors(&mut self, ctx: &Context) {
        // Show errors
        Window::new("Errors")
            .open(&mut self.errors.show)
            .show(ctx, |ui| {
                if self.errors.buffer.is_empty() {
                    ui.label("No errors");
                } else {
                    self.errors.buffer.retain(|&index, error| {
                        ui.horizontal(|ui| {
                            ui.label(self.files[index].display().to_string())
                                .on_hover_text(error.to_string());
                            !ui.button("🗙").clicked()
                        })
                        .inner
                    });
                }
            });
    }

    fn files(&mut self, ctx: &Context) {
        // Show files (if any):
        if !self.files.is_empty() {
            let mut open = true;
            Window::new("Files")
                .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
                .open(&mut open)
                .show(ctx, |ui| {
                    self.files.retain(with_index(|index, file: &DroppedFile| {
                        ui.horizontal(|ui| {
                            let mut include = !self.filter.contains(&index);
                            if ui.checkbox(&mut include, "").changed() {
                                if include {
                                    self.filter.remove(&index);
                                } else {
                                    self.filter.insert(index);
                                }
                            }
                            ui.label(file.display().to_string());
                            ui.color_edit_button_srgba(&mut self.colors[index]);
                            !ui.button("🗙").clicked()
                        })
                        .inner
                    }));
                });
            if !open {
                self.files.clear();
            }
        }
    }
}

impl App {
    fn svg(&mut self, ui: &mut Ui) -> Result<Vec<u8>> {
        // let mut buf = String::new();
        let mut buf = Vec::new();
        {
            // let size = (1920, 1080);
            let drawing_area =
                BitMapBackend::with_buffer(&mut buf, self.svg.size).into_drawing_area();
            // let drawing_area = SVGBackend::with_string(&mut buf, self.svg.size).into_drawing_area();
            drawing_area.fill(&WHITE)?;
            // Labels
            for label in &*self.labels {
                drawing_area.draw_text(
                    &label.text,
                    &("Arial", self.visuals.x.labeled.font_size)
                        .into_text_style(&self.svg.size)
                        .color(&BLACK),
                    (label.coordinates.x as _, label.coordinates.y as _),
                )?;
            }
            // Points
            for point in &*self.points {
                drawing_area.draw(&Circle::new(
                    (point.coordinates.x as _, point.coordinates.y as _),
                    point.size,
                    ShapeStyle {
                        color: RGBAColor(
                            point.color.r(),
                            point.color.g(),
                            point.color.b(),
                            point.color.a() as f64 / u8::MAX as f64,
                        ),
                        filled: point.filled,
                        stroke_width: default(),
                    },
                ))?;
                // drawing_area.draw(&Circle::new(
                //     (point.coordinates.x, point.coordinates.y),
                //     point.radius,
                //     original_style,
                // ));
            }
            let mut chart = ChartBuilder::on(&drawing_area)
                .x_label_area_size(self.x_label_area_size)
                .y_label_area_size(self.y_label_area_size)
                .margin(self.margin)
                .caption("Header", ("Ubuntu", 50.0))
                .build_cartesian_2d(
                    self.bounds.x.clone().into_segmented(),
                    self.bounds.y.start as _..self.bounds.y.end as f64,
                )?;
            chart
                .configure_mesh()
                .disable_mesh()
                .x_desc(&self.mesh.x_desc)
                .y_desc(&self.mesh.y_desc)
                .x_labels(100)
                .y_label_offset(self.test)
                .set_all_tick_mark_size(RelativeSize::Width(self.tick_mark_size))
                .axis_style(BLACK.stroke_width(self.stroke_width))
                .axis_desc_style(("sans-serif", 20))
                .draw()?;
            // chart.configure_series_labels().draw()?;

            // Parsed
            let parsed = &self.parsed[&0];
            // Bounded
            let peaks = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<Bounded>()
                    .get((&parsed.peaks, &self.bounds))
            });
            // Normalized
            let peaks = ui.memory_mut(|memory| memory.caches.cache::<Normalized>().get(&peaks));
            chart.draw_series(
                Histogram::vertical(&chart)
                    .style(BLACK.filled())
                    .margin(self.margin1)
                    // .data(
                    //     data.iter()
                    //         .enumerate()
                    //         .map(|(index, &x)| (index as u64, x as f64)),
                    // ),
                    // .data(data.iter().map(|(&x, &y)| (x, y))),
                    .data(peaks),
            )?;
            // To avoid the IO failure being ignored silently, we manually call the present function
            drawing_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
        }
        Ok(buf)
    }

    // Bar Chart
    fn bar_chart(&self, ui: &mut Ui) -> BarChart {
        // Parsed
        let parsed = &self.parsed[&0];
        // Bounded
        let peaks = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<Bounded>()
                .get((&parsed.peaks, &self.bounds))
        });
        // Normalized
        let peaks = ui.memory_mut(|memory| memory.caches.cache::<Normalized>().get(&peaks));
        // Bar chart
        let bars = peaks
            .iter()
            .map(|(&mass, &intensity)| Bar::new(mass as _, intensity as _).fill(COLOR))
            .collect();
        BarChart::new(bars)
            .color(COLOR)
            .width(self.visuals.chart.width)
    }

    // Labels
    fn labels(&self, texts: &mut Vec<Text>) {
        for label in &*self.labels {
            texts.push(
                Text::new(
                    PlotPoint::new(label.coordinates.x, label.coordinates.y),
                    RichText::new(&label.text).font(FontId::new(
                        self.visuals.x.labeled.font_size,
                        FontFamily::Name(
                            label.bold.then_some("Arial Bold").unwrap_or("Arial").into(),
                        ),
                    )),
                )
                .anchor(Align2::CENTER_TOP)
                .color(COLOR),
            );
        }
    }

    // Points
    fn points(&self, points: &mut Vec<Points>) {
        for point in &*self.points {
            points.push(
                Points::new(vec![[point.coordinates.x, point.coordinates.y]])
                    .color(point.color)
                    .filled(point.filled)
                    .radius(point.size),
            );
        }
    }

    fn axes(&self, lines: &mut Vec<Line>, texts: &mut Vec<Text>) {
        self.x(lines, texts);
        self.y(lines, texts);
    }

    fn x(&self, lines: &mut Vec<Line>, texts: &mut Vec<Text>) {
        let y = self.bounds.y.start as _;
        lines.push(
            DLine::new(vec![
                [self.bounds.x.start as _, y],
                [self.bounds.x.end as _, y],
            ])
            .color(COLOR)
            .name("x")
            .width(self.visuals.x.labeled.width)
            .into(),
        );
        for index in self.bounds.x().step_by(self.visuals.x.unlabeled.step as _) {
            let x = index as _;
            lines.push(
                DLine::new(vec![[x, y], [x, y - self.visuals.x.unlabeled.height]])
                    .color(COLOR)
                    .width(self.visuals.x.unlabeled.width)
                    .into(),
            );
        }
        for index in self
            .bounds
            .x()
            .filter(|x| x % self.visuals.x.labeled.step == 0)
        {
            let x = index as _;
            lines.push(
                DLine::new(vec![[x, y], [x, y - self.visuals.x.labeled.height]])
                    .color(COLOR)
                    .width(self.visuals.x.labeled.width)
                    .into(),
            );
            texts.push(
                Text::new(
                    PlotPoint::new(x, y - self.visuals.x.labeled.height),
                    RichText::new(index.to_string()).font(FontId::new(
                        self.visuals.x.labeled.font_size,
                        FontFamily::Name("Arial".into()),
                    )),
                )
                .anchor(Align2::CENTER_TOP)
                .color(COLOR),
            );
        }
    }

    fn y(&self, lines: &mut Vec<Line>, texts: &mut Vec<Text>) {
        let x = self.bounds.x.start as _;
        lines.push(
            DLine::new(vec![
                [x, self.bounds.y.start as _],
                [x, self.bounds.y.end as _],
            ])
            .color(COLOR)
            .name("y")
            .width(self.visuals.y.width)
            .into(),
        );
        for index in self.bounds.y().filter(|y| y % self.visuals.y.step == 0) {
            let y = index as _;
            lines.push(
                DLine::new(vec![[x, y], [x - self.visuals.y.height, y]])
                    .color(COLOR)
                    .width(self.visuals.y.width)
                    .into(),
            );
            texts.push(
                Text::new(
                    PlotPoint::new(x - self.visuals.y.height - 1.0, y),
                    RichText::new(index.to_string()).font(FontId::new(
                        self.visuals.x.labeled.font_size,
                        FontFamily::Name("Arial".into()),
                    )),
                )
                .anchor(Align2::RIGHT_CENTER)
                .color(COLOR),
            );
        }
    }
}

impl eframe::App for App {
    #[allow(unsafe_code)]
    fn post_rendering(&mut self, window_size_px: [u32; 2], frame: &Frame) {
        if !self.save_plot {
            return;
        }
        error!(?window_size_px);

        self.save_plot = false;
        if let Some(context) = frame.gl() {
            let [window_width, window_height] = window_size_px;

            // We needed the relative values here, because we need to have them
            // in relation to the screen_size_px.
            // Calculating with absolut px values does not always work. Using
            // relative values, we have a working solution for all cases
            // let x = self.plot_size[0] * window_width as f32;
            // let y = self.plot_size[1] * window_height as f32;
            // let width = self.plot_size[2] * window_width as f32;
            // let height = self.plot_size[3] * window_height as f32;
            let x = 0.0;
            let y = 0.0;
            let width = window_width as f32;
            let height = window_height as f32;
            error!(?x, ?y, ?width, ?height);

            let mut buf = vec![0u8; width as usize * height as usize * 4];
            let pixels = PixelPackData::Slice(&mut buf[..]);
            unsafe {
                context.read_pixels(
                    x as i32,
                    y as i32,
                    width as i32,
                    height as i32,
                    RGBA,
                    UNSIGNED_BYTE,
                    pixels,
                );
            }

            // Flip vertically:
            let mut rows: Vec<Vec<u8>> = buf
                .chunks(width as usize * 4)
                .into_iter()
                .map(|chunk| chunk.to_vec())
                .collect();
            rows.reverse();
            let buf: Vec<u8> = rows.into_iter().flatten().collect();
            self.plot_to_save = Some(ColorImage::from_rgba_unmultiplied(
                [width as _, height as _],
                &buf[..],
            ));
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.top_panel(ctx, frame);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
        // self.windows(ctx);
        self.drag_and_drop_files(ctx);
        self.errors(ctx);
        self.files(ctx);
    }
}

/// Line
enum Line {
    Horizontal(HLine),
    Vertical(VLine),
    Diagonal(DLine),
}

impl From<HLine> for Line {
    fn from(value: HLine) -> Self {
        Self::Horizontal(value)
    }
}

impl From<VLine> for Line {
    fn from(value: VLine) -> Self {
        Self::Vertical(value)
    }
}

impl From<DLine> for Line {
    fn from(value: DLine) -> Self {
        Self::Diagonal(value)
    }
}

/// Bounds
#[derive(Clone, Debug, Default, Deserialize, Hash, Serialize)]
struct Bounds {
    x: Range<u64>,
    y: Range<u64>,
}

impl Bounds {
    fn x(&self) -> RangeInclusive<u64> {
        self.x.start..=self.x.end
    }

    fn y(&self) -> RangeInclusive<u64> {
        self.y.start..=self.y.end
    }
}

/// Center
trait Center {
    fn center(&self) -> f64;
}

impl Center for Range<u64> {
    fn center(&self) -> f64 {
        self.start as f64 + self.end as f64 / 2.0
    }
}

/// Errors
#[derive(Debug, Default)]
struct Errors {
    show: bool,
    buffer: IndexMap<usize, Error>,
}

/// Label
#[derive(Default, Deserialize, Serialize)]
struct Label {
    text: String,
    bold: bool,
    coordinates: Coordinates,
}

/// Label
#[derive(Default, Deserialize, Serialize)]
struct Point {
    // shape: MarkerShape,
    color: Color32,
    filled: bool,
    size: f32,
    coordinates: Coordinates,
}

enum Shape {
    Circle,
    Diamond,
    Square,
    Cross,
    Plus,
    Up,
    Down,
    Left,
    Right,
    Asterisk,
}

/// Coordinates
#[derive(Default, Deserialize, Serialize)]
struct Coordinates {
    x: f64,
    y: f64,
}

/// Visuals
#[derive(Deserialize, Serialize)]
struct Visuals {
    data_aspect: f32,
    view_aspect: f32,
    width: f32,
    height: f32,
    chart: Chart,
    x: Division,
    y: Axes,
}

impl Default for Visuals {
    fn default() -> Self {
        Self {
            data_aspect: 1.0,
            view_aspect: 1.0,
            width: default(),
            height: default(),
            chart: default(),
            x: default(),
            y: default(),
        }
    }
}

#[derive(Default, Deserialize, Serialize)]
struct Chart {
    width: f64,
}

#[derive(Default, Deserialize, Serialize)]
struct Division {
    unlabeled: Unlabeled,
    labeled: Axes,
}

#[derive(Default, Deserialize, Serialize)]
struct Unlabeled {
    height: f64,
    width: f32,
    step: u64,
}

#[derive(Default, Deserialize, Serialize)]
struct Axes {
    font_size: f32,
    height: f64,
    width: f32,
    step: u64,
}

// mod font {
//     use egui::FontFamily;
//     use serde::{Deserialize, Serialize};

//     pub(super) struct Font {
//         pub(super) family: Family,
//         pub(super) style: Style,
//     }

//     impl From<Font> for FontFamily {
//         fn from(value: Font) -> Self {
//             value.style.format("Arial")
//             match value.family {
//                 Family::Arial => Self::Name("Arial".into()),
//                 Family::Helvetica => Self::Name("Helvetica".into()),
//             }
//         }
//     }

//     #[derive(Clone, Copy, Default, Deserialize, Serialize)]
//     enum Family {
//         #[default]
//         Arial,
//         Helvetica,
//     }

//     #[derive(Clone, Copy, Default, Deserialize, Serialize)]
//     pub(super) enum Style {
//         #[default]
//         Regular,
//         Bold,
//     }

//     impl Style {
//         fn format(name: )
//     }
// }

mod bounder;
mod normalizer;
