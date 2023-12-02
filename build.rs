// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! This `build.rs` file generates the `README.md` file from `docs.md`.

use std::fmt::{Display, Formatter};
use std::{fs, io};

/// The path to the input file.
const INPUT: &str = "docs.md";
/// The path to the output file.
const OUTPUT: &str = "README.md";

const COPYRIGHT: &str = "\
<!-- This Source Code Form is subject to the terms of the Mozilla Public
   - License, v. 2.0. If a copy of the MPL was not distributed with this
   - file, You can obtain one at https://mozilla.org/MPL/2.0/. --> \
";

const NOTE: &str = "\
<!-- This `README.md` file is automatically generated from `docs.md`, which uses `rustdoc`'s syntax
   - to provide documentation for the `#[cfg_attrs { ... }]` macro too.
   -
   - See `build.rs` if you're interested to see the code, or edit `docs.md` to edit the
   - documentation. --> \
";

const HEADER: &str = "# `#[cfg_attrs { ... }]`";

fn main() -> io::Result<()> {
	// If `docs.md` is changed, rerun the build script.
	println!("cargo:rerun-if-changed={INPUT}");
	println!("cargo:rerun-if-changed={OUTPUT}");

	let input = fs::read_to_string(INPUT)?;

	let copyright_lines = COPYRIGHT.lines().count() + 1;

	let docs: Doc<'_> = input.lines().skip(copyright_lines).collect();

	fs::write(OUTPUT, docs.to_string())?;

	Ok(())
}

struct Doc<'lines> {
	nodes: Vec<Node<'lines>>,
}

impl<'lines> FromIterator<&'lines str> for Doc<'lines> {
	fn from_iter<T: IntoIterator<Item = &'lines str>>(lines: T) -> Self {
		let mut code_block = None;
		let mut nodes = Vec::new();

		const INDENTATION: usize = 3;

		let code_indentation = |line: &str| {
			let mut indentation = 0;

			for r#char in line.chars().take(INDENTATION) {
				if r#char == ' ' {
					indentation += 1;
				} else {
					break;
				}
			}

			if indentation == INDENTATION {
				if let Some(r#char) = line.chars().nth(INDENTATION) {
					if r#char.is_whitespace() {
						return None;
					}
				}
			}

			Some(indentation)
		};

		for line in lines {
			if let Some(CodeBlock {
				backticks,
				indentation,
				lines,
				..
			}) = &mut code_block
			{
				if code_indentation(line).map_or(false, |indent| indent < *indentation) && !line.is_empty() {
					// End of the code block.

					nodes.push(Node::CodeBlock(code_block.take().unwrap()));
				} else {
					if line.len() > *indentation && line[*indentation..].trim_end() == *backticks {
						// End of the code block.

						nodes.push(Node::CodeBlock(code_block.take().unwrap()));
					} else {
						lines.push(line.get(*indentation..).unwrap_or(""));
					}

					continue;
				}
			}

			if let Some(indentation) = code_indentation(line) {
				let mut backticks = 0;

				for r#char in line.chars().skip(indentation) {
					if r#char == '`' {
						backticks += 1;
					} else {
						break;
					}
				}

				if backticks >= 3 {
					code_block = Some(CodeBlock {
						backticks: &line[indentation..(indentation + backticks)],
						indentation,

						info: {
							let info = line.get((indentation + backticks)..).map(|info| info.trim_end());

							info.filter(|info| !info.is_empty())
						},

						lines: Vec::new(),
					});

					continue;
				}
			}

			nodes.push(Node::Line(process_heading(line)));
		}

		Doc { nodes }
	}
}

enum Node<'lines> {
	Line(String),
	CodeBlock(CodeBlock<'lines>),
}

/// Represents a code block in the Markdown file.
struct CodeBlock<'lines> {
	backticks: &'lines str,
	indentation: usize,
	info: Option<&'lines str>,
	lines: Vec<&'lines str>,
}

/// Processes the `#`-hiding of lines within a Rust code block.
fn process_hiding(line: &str, indentation: usize) -> Option<String> {
	if line.len() > indentation {
		let trim = line[indentation..].trim_end();

		if trim == "#" || trim.starts_with("# ") {
			// If it starts with `#`, it is hidden.
			return None;
		} else if trim == "##" || trim.starts_with("## ") {
			// If it starts with `##`, then remove one of those `#`s.
			return Some(format!("{}{}", &line[..indentation], &line[(indentation + 1)..]));
		}
	}

	// Otherwise, it's just a normal line.
	Some(line.to_owned())
}

/// Add an extra `#` to each heading.
fn process_heading(line: &str) -> String {
	const LEVELS: usize = 6;

	let mut levels = 0;

	for r#char in line.chars().take(LEVELS) {
		if r#char == '#' {
			levels += 1;
		} else {
			break;
		}
	}

	if levels >= 1 && line.chars().nth(levels) == Some(' ') {
		// Heading - insert an extra `#`.

		format!("#{}", line)
	} else {
		// Not heading.

		line.to_owned()
	}
}

impl<'lines> Display for CodeBlock<'lines> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let ws: String = " ".repeat(self.indentation);
		let backticks = self.backticks;

		writeln!(f, "{ws}{backticks}{}", self.info.unwrap_or("rust"))?;

		for line in &self.lines {
			if let Some(line) = process_hiding(line, self.indentation) {
				writeln!(f, "{}", line)?;
			}
		}

		write!(f, "{ws}{backticks}")?;

		Ok(())
	}
}

impl<'lines> Display for Node<'lines> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::CodeBlock(code_block) => write!(f, "{}", code_block),
			Self::Line(line) => write!(f, "{}", line),
		}
	}
}

impl<'lines> Display for Doc<'lines> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "{}", COPYRIGHT)?;
		writeln!(f)?;
		writeln!(f, "{}", NOTE)?;
		writeln!(f)?;
		writeln!(f, "{}", HEADER)?;

		for node in &self.nodes {
			writeln!(f, "{}", node)?;
		}

		Ok(())
	}
}
