/// Example demonstrating triangulation-based pathfinding for non-axis-aligned polygons
///
/// This example shows how to use the triangulation feature for pathfinding
/// in rotated or irregularly shaped polygons.
///
/// Run with: cargo run --example triangulation_demo

fn main() {
    println!("=== Triangulation-Based Pathfinding Demo ===\n");

    println!("This example demonstrates the triangulation feature.");
    println!("The actual implementation is in the test suite.\n");

    println!("Key Features:");
    println!("  1. Ear clipping triangulation for polygon decomposition");
    println!("  2. Automatic detection of axis-aligned vs. rotated polygons");
    println!("  3. Efficient pathfinding through non-rectangular spaces\n");

    println!("Usage Examples:");
    println!("  // For rotated or irregular polygons:");
    println!("  let poly = Polygon::from(points);");
    println!("  let array = poly.as_bounded_block_array_triangulated();");
    println!("  let path = array.find_displacement(start, end);\n");

    println!("  // Auto-detect and choose best method:");
    println!("  let array = poly.as_bounded_block_array_auto();\n");

    println!("Run tests to see it in action:");
    println!("  cargo test triangulation");
    println!("  cargo test displacement_route\n");

    println!("=== Demo Complete ===");
}
