// #![feature(const_trait_impl)]
// #![feature(let_chains)]
// #![feature(is_some_and)]
// #![feature(iter_intersperse)]
#![feature(decl_macro)]
#![feature(default_free_fn)]
#![feature(adt_const_params)]

use self::app::App;

// 𝍠C𝍡C𝍢C𝍣C𝍤
// 𝍩𝍪𝍫𝍬𝍭

// When compiling natively.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

// When compiling to web.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();
    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id",
            web_options,
            Box::new(|cc| Box::new(App::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}

mod app;
mod parser;
mod utils;

mod tests {
    // use uom::si::{
    //     f64::Length,
    //     length::{centimeter, inch, meter},
    // };

    use uom::si::{
        f64::Length,
        length::{centimeter, inch, meter},
        reciprocal_length::{reciprocal_centimeter, reciprocal_meter},
    };

    #[test]
    fn test() {
        let l = Length::new::<inch>(1.0);
        let dpi = 1200.0 / l;
        println!("{:?}", dpi);
        println!("{:?}", dpi.get::<reciprocal_centimeter>());
    }
}
