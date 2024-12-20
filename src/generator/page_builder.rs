
use std::collections::HashSet;
use comrak::markdown_to_html;
use maud::{html, Markup, DOCTYPE};
use serde_json::json;

use super::{Post, PostPreview};


fn navbar(posts: &[PostPreview], tag_set: &HashSet<String>) -> Markup {
    let previews = json!(posts).to_string();

    html! {
        nav class="navbar" {
            div class="container" {
                // Left: Logo or brand name
                // a href="/" class="navbar-brand" { "ZG" }

                // Center: Navigation links
                ul class="navbar-navlinks" {
                    li class="nav-brand" { a href="/" { "ZG" } }
                    li class="nav-item" { a href="/" { "Home" } }
                    li class="nav-item" { a href="/archive" { "Archive" } }
                    li class="nav-item" { a href="/about" { "About" } }
                }

                // Right: Search bar
				div class="navbar-search" id="search-form" tabindex="0" {
					input class="search-input" type="search" placeholder="Search" aria-label="Search" id="search-input";

                    // Tag filter section (Initially hidden)
                    div class="tag-filter" id="tag-filter" {
                        p class="label" { "Filter by Tag:" }
                        div class="tag-div" {
                            form class="tag-form" {
                                // Checkboxes for each tag
                                @for tag in tag_set {
                                    label class="tag-box" for={ ("checkbox-".to_string() + &tag) } {
                                        input type="checkbox" id={ ("checkbox-".to_string() + &tag) } name="tags" value={(tag.clone())} {
                                        }
                                        span class="tag-term" {
                                            (tag)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div class="search-results" {
                        p class="label" { "Results: " }
                        div class="results-div" {
                            
                        }
                    }
                }
                // This makes previews available to the client.
                script {
                    (maud::PreEscaped(format!("const previews = {};", previews)))
                }
                // Later potentially add "defer" to this script because it's not
                // immediately necessary.
                script src="assets/searchbar.js" {}
            }
        }
    }
}


fn header(current_post: &Post, posts: &[PostPreview], tag_set: &HashSet<String>) -> Markup {
    html! {
        // Metadata
        head {
            meta charset="utf-8";
            title { (current_post.parsed_post_data.data.title) }
            meta name="author" content=(current_post.parsed_post_data.data.author );
            meta name="description" content=(current_post.parsed_post_data.data.description);
            link rel="stylesheet" type="text/css" href="assets/styles.css";
        }
        header {
            // navbar
            (navbar(posts, tag_set))
        }
    }
}

fn body(blog_post: String) -> Markup {
    html! {
        body {
            div.container {
                (maud::PreEscaped(blog_post))
            }
        }
    }
}

fn footer(publishing_date: &str) -> Markup {
	html! {
		footer {
			p {
				"Published on: " (publishing_date)
			}
			a href="#top" {
				"Back to top"
			}
		}
	}
}

pub fn generate_blog_post(current_post: &Post, previews: &[PostPreview], tag_set: &HashSet<String>) -> Markup {
    // Markdown options and conversion to HTML
    let options = comrak::ComrakOptions::default();
    let content = markdown_to_html(&current_post.parsed_post_data.content, &options);
    let publishing_date = &current_post.parsed_post_data.data.date;
    
    // Generate the blog post page
    html! {
        (DOCTYPE)
        html {
            (header(current_post, previews, tag_set))
            (body(content))
            (footer(publishing_date))
        }
    }
}

