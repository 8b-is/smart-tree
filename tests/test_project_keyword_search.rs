//! Test keyword-based project search functionality

use std::path::PathBuf;

#[test]
fn test_keyword_search_finds_q8_caster() {
    // This test verifies that keyword search can find projects
    // Use case: Search for "cast" and "tv" should find q8-caster

    let test_path = PathBuf::from("/aidata/ayeverse");
    if !test_path.exists() {
        eprintln!("Test path doesn't exist, skipping test");
        return;
    }

    // Import the ProjectsFormatter
    use st::formatters::projects::ProjectsFormatter;

    let formatter = ProjectsFormatter::new();
    let projects = formatter
        .scan_projects(&test_path)
        .expect("Failed to scan projects");

    println!("Found {} total projects", projects.len());

    // Filter by keywords: "cast" or "tv"
    let keywords = vec!["cast".to_string(), "tv".to_string()];
    let matching_projects: Vec<_> = projects
        .iter()
        .filter(|proj| {
            let searchable = format!(
                "{} {} {} {}",
                proj.name.to_lowercase(),
                proj.summary.to_lowercase(),
                proj.dependencies.join(" ").to_lowercase(),
                proj.path.display().to_string().to_lowercase()
            );

            keywords.iter().any(|kw| searchable.contains(kw))
        })
        .collect();

    println!("\nProjects matching keywords {:?}:", keywords);
    for proj in &matching_projects {
        println!("  - {} ({})", proj.name, proj.path.display());
        println!("    Summary: {}", proj.summary);
    }

    // Verify q8-caster was found
    let found_q8_caster = matching_projects.iter().any(|p| {
        p.name.contains("q8-caster") || p.path.display().to_string().contains("q8-caster")
    });

    assert!(
        found_q8_caster,
        "Expected to find q8-caster project with keywords 'cast' or 'tv'"
    );

    println!("\n✓ Test passed: Found q8-caster with keyword search!");
}
