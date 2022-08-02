use std::path::PathBuf;

use graphpu::{
    bootstrap::{start_frame, ConfigBuilder},
    MainApp,
};

pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let dataset_folder =  PathBuf::from(ROOT).join("examples").join("dataset");
    let test_node_data = dataset_folder.join("test_data_node.csv");
    let test_edge_data = dataset_folder.join("small_data_edge.csv");

    let config_builder = ConfigBuilder::default().app_creator(Box::new(move |cc| {
        let mut app = MainApp::new(cc);
        app.models.load_data(
            test_node_data.to_str().unwrap(),
            test_edge_data.to_str().unwrap(),
            0,
            1,
        ).unwrap();
        Box::new(app)
    }));
    start_frame(config_builder.build());
}