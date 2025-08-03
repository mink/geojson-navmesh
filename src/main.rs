use geojson::GeoJson;

fn main() {
    let input = "input.geojson";
    let output = "output.geojson";

    let contents = std::fs::read_to_string(input)
        .expect("failed to read input");

    let geojson = contents.parse::<GeoJson>().expect("invalid GeoJSON");

    let collection = match geojson {
        GeoJson::FeatureCollection(collection) => collection,
        _ => {
            eprintln!("must be a FeatureCollection");
            std::process::exit(1);
        }
    };

    let mut features = Vec::new();

    for (_, feature) in collection.features.iter().enumerate() {
        // todo: generate navmesh
        features.push(feature.clone());
    }

    let collection = geojson::FeatureCollection {
        bbox: collection.bbox,
        features,
        foreign_members: collection.foreign_members,
    };

    std::fs::write(output, GeoJson::FeatureCollection(collection).to_string())
        .expect("failed to write output");
}
