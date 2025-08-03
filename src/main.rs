use geojson::{Feature, GeoJson, Geometry, Value};
use i_triangle::float::triangulatable::Triangulatable;
use i_triangle::float::triangulation::Triangulation;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: {} <input> <output>", args[0]);
        std::process::exit(1);
    }
    let input = &args[0];
    let output = &args[1];

    let contents = std::fs::read_to_string(input)
        .expect("failed to read input");

    let geojson = contents.parse::<GeoJson>().expect("invalid GeoJSON");

    let collection = match geojson {
        GeoJson::FeatureCollection(collection) => {
            let mut features: Vec<Feature> = Vec::new();
            for feature in collection.features {
                if let Some(geom) = feature.geometry {
                    extract_tessellated_features(&geom, &mut features);
                }
            }
            geojson::FeatureCollection {
                bbox: collection.bbox,
                features,
                foreign_members: collection.foreign_members,
            }
        },
        _ => {
            eprintln!("must be a FeatureCollection");
            std::process::exit(1);
        }
    };

    std::fs::write(output, GeoJson::FeatureCollection(collection).to_string())
        .expect("failed to write output");
}

fn extract_tessellated_features(geom: &Geometry, features: &mut Vec<Feature>) {
    match &geom.value {
        Value::Polygon(rings) => tessellate_polygon_to_features(rings, features),
        Value::MultiPolygon(polygons) => {
            for (_, rings) in polygons.iter().enumerate() {
                tessellate_polygon_to_features(rings, features);
            }
        }
        _ => eprintln!("unsupported geometry: {:?}", geom.value),
    }
}

fn tessellate_polygon_to_features(rings: &Vec<Vec<Vec<f64>>>, features: &mut Vec<Feature>) {
    // todo: configurable
    let min_area = 0.5;
    let chunk_size: usize = 3;

    let shape: Vec<Vec<[f64; 2]>> = rings
        .iter()
        .map(|ring| ring.iter().map(|p| [p[0], p[1]]).collect())
        .collect();

    let tessellation: Triangulation<[f64; 2], u16> = shape
        .triangulate()
        .into_delaunay()
        .refine_with_circumcenters_by_obtuse_angle(min_area) // steiner points
        .to_triangulation();

    for triangle in tessellation.indices.chunks(chunk_size) {
        let coords: Vec<Vec<f64>> = triangle
            .iter()
            .map(|&i| vec![tessellation.points[i as usize][0], tessellation.points[i as usize][1]])
            .collect();

        let mut ring = coords.clone();
        ring.push(coords[0].clone());

        let feature = Feature {
            geometry: Some(Geometry::new(Value::Polygon(vec![ring]))),
            properties: None,
            id: None,
            bbox: None,
            foreign_members: None,
        };

        features.push(feature);
    }
}
