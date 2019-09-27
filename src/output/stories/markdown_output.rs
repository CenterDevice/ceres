use output::stories::*;

use handlebars::Handlebars;
use serde_json;

pub struct MarkDownOutputStory;

impl OutputStory for MarkDownOutputStory {
    fn output<T: Write>(&self, writer: &mut T, story: &Story) -> Result<()> {
        let mut reg = Handlebars::new();

        let template = include_str!("../../../includes/stories.export.markdown.hbs");

        let md = reg.render_template(template, story)
            .chain_err(|| ErrorKind::OutputFailed)?;
        writer.write(md.as_bytes()).chain_err(|| ErrorKind::OutputFailed)?;

        Ok(())
    }
}

