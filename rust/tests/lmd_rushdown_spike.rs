//! Phase-0 rushdown extension spike (spec §8.1).
//!
//! This file is the go/no-go acceptance test for the lmd "extension path":
//! can we express lmd's custom directives directly as rushdown 0.18
//! parser+renderer extensions, instead of a separate preprocessor stage?
//!
//! It proves two things end-to-end against the *real* rushdown 0.18 API:
//!   (a) a custom `@`-prefixed BLOCK directive  (`@upper hello world`)
//!   (b) a `{{ }}` INLINE directive            (`{{ shout:done }}`)
//!
//! Both are implemented as a custom AST node + a parser extension (parses
//! syntax -> AST node) + a renderer extension (renders node -> HTML), wired
//! through `new_markdown_to_html(parser_opts, html_opts, parser_ext, renderer_ext)`
//! which returns a closure `(&mut String, &str) -> Result<_>`.
//!
//! If this spike PASSES, Phase 1 takes the extension path.
//! Fallback if it had proven impossible: a preprocessor stage that rewrites
//! the directives into plain HTML/markdown before handing off to rushdown.


use core::{
    any::TypeId,
    fmt::{self, Write},
};

use rushdown::{
    as_extension_data,
    ast::*,
    new_markdown_to_html, parser,
    parser::*,
    renderer,
    renderer::{
        html::{renderer_extension, Options, RendererExtension},
        *,
    },
    text,
    text::*,
    Result,
};

// (a) UpperBlock — custom `@upper ...` BLOCK directive {{{

/// Leaf-block node holding the already-uppercased remainder of an `@upper`
/// line. We store an owned `String` (not a `text::Value` slice) because the
/// rendered text is a transformation of the source, not a verbatim span.
#[derive(Debug)]
struct UpperBlock {
    text: String,
}

impl UpperBlock {
    fn new(text: String) -> Self {
        Self { text }
    }
}

impl NodeKind for UpperBlock {
    fn typ(&self) -> NodeType {
        NodeType::LeafBlock
    }

    fn kind_name(&self) -> &'static str {
        "UpperBlock"
    }
}

impl PrettyPrint for UpperBlock {
    fn pretty_print(&self, w: &mut dyn Write, _source: &str, level: usize) -> fmt::Result {
        writeln!(w, "{}UpperBlock: {}", pp_indent(level), self.text)
    }
}

impl From<UpperBlock> for KindData {
    fn from(e: UpperBlock) -> Self {
        KindData::Extension(Box::new(e))
    }
}

const UPPER_PREFIX: &[u8] = b"@upper ";

#[derive(Debug, Default)]
struct UpperBlockParser {}

impl UpperBlockParser {
    fn new() -> Self {
        Self {}
    }
}

impl BlockParser for UpperBlockParser {
    fn trigger(&self) -> &[u8] {
        b"@"
    }

    fn open(
        &self,
        arena: &mut Arena,
        _parent_ref: NodeRef,
        reader: &mut text::BasicReader,
        _ctx: &mut parser::Context,
    ) -> Option<(NodeRef, State)> {
        let (line, _seg) = reader.peek_line_bytes()?;
        // Only fire on `@upper ` lines; otherwise leave the text untouched.
        if !line.starts_with(UPPER_PREFIX) {
            return None;
        }
        // Remainder after the prefix, trimmed of the trailing newline.
        let rest = &line[UPPER_PREFIX.len()..];
        let rest = rest
            .strip_suffix(b"\n")
            .or_else(|| rest.strip_suffix(b"\r"))
            .unwrap_or(rest);
        let upper = String::from_utf8_lossy(rest).to_uppercase();
        reader.advance_to_eol();
        Some((arena.new_node(UpperBlock::new(upper)), State::NO_CHILDREN))
    }

    fn cont(
        &self,
        _arena: &mut Arena,
        _node_ref: NodeRef,
        _reader: &mut text::BasicReader,
        _ctx: &mut parser::Context,
    ) -> Option<State> {
        None
    }

    fn can_interrupt_paragraph(&self) -> bool {
        true
    }
}

impl From<UpperBlockParser> for AnyBlockParser {
    fn from(p: UpperBlockParser) -> Self {
        AnyBlockParser::Extension(Box::new(p))
    }
}

struct UpperBlockHtmlRenderer<W: TextWrite> {
    _phantom: core::marker::PhantomData<W>,
    writer: html::Writer,
}

impl<W: TextWrite> UpperBlockHtmlRenderer<W> {
    fn with_options(html_opts: Options, _options: NoRendererOptions) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            writer: html::Writer::with_options(html_opts),
        }
    }
}

impl<W: TextWrite> RenderNode<W> for UpperBlockHtmlRenderer<W> {
    fn render_node<'a>(
        &self,
        w: &mut W,
        _source: &'a str,
        arena: &'a Arena,
        node_ref: NodeRef,
        entering: bool,
        _context: &mut renderer::Context,
    ) -> Result<WalkStatus> {
        if entering {
            self.writer.write_safe_str(w, "<p>")?;
            let ub = as_extension_data!(arena, node_ref, UpperBlock);
            self.writer.write_html(w, ub.text.as_str())?;
        } else {
            self.writer.write_safe_str(w, "</p>\n")?;
        }
        Ok(WalkStatus::Continue)
    }
}

impl<'cb, W> NodeRenderer<'cb, W> for UpperBlockHtmlRenderer<W>
where
    W: TextWrite + 'cb,
{
    fn register_node_renderer_fn(self, nrr: &mut impl NodeRendererRegistry<'cb, W>) {
        nrr.register_node_renderer_fn(TypeId::of::<UpperBlock>(), BoxRenderNode::new(self));
    }
}

fn upper_block_parser_extension() -> impl ParserExtension {
    parser_extension(|p| {
        // Lower priority value = earlier; sit ahead of paragraph so `@upper`
        // lines are claimed before becoming plain paragraphs.
        p.add_block_parser(UpperBlockParser::new, NoParserOptions, PRIORITY_ATX_HEADING);
    })
}

fn upper_block_html_renderer_extension<'cb, W>() -> impl RendererExtension<'cb, W>
where
    W: TextWrite + 'cb,
{
    renderer_extension(|r| {
        r.add_node_renderer(UpperBlockHtmlRenderer::with_options, NoRendererOptions);
    })
}

// (a) UpperBlock }}}

// (b) ShoutInline — custom `{{ shout:TEXT }}` INLINE directive {{{

/// Inline node holding the `TEXT` captured from `{{ shout:TEXT }}`.
#[derive(Debug)]
struct ShoutInline {
    text: text::Value,
}

impl ShoutInline {
    fn new(text: impl Into<text::Value>) -> Self {
        Self { text: text.into() }
    }
}

impl NodeKind for ShoutInline {
    fn typ(&self) -> NodeType {
        NodeType::Inline
    }

    fn kind_name(&self) -> &'static str {
        "ShoutInline"
    }
}

impl PrettyPrint for ShoutInline {
    fn pretty_print(&self, w: &mut dyn Write, source: &str, level: usize) -> fmt::Result {
        writeln!(w, "{}Shout: {}", pp_indent(level), self.text.str(source))
    }
}

impl From<ShoutInline> for KindData {
    fn from(e: ShoutInline) -> Self {
        KindData::Extension(Box::new(e))
    }
}

#[derive(Debug, Default)]
struct ShoutInlineParser {}

impl ShoutInlineParser {
    fn new() -> Self {
        Self {}
    }
}

impl InlineParser for ShoutInlineParser {
    fn trigger(&self) -> &[u8] {
        b"{"
    }

    fn parse(
        &self,
        arena: &mut Arena,
        _parent_ref: NodeRef,
        reader: &mut text::BlockReader,
        _ctx: &mut parser::Context,
    ) -> Option<NodeRef> {
        let (line, seg) = reader.peek_line_bytes()?;
        // Expect `{{ shout:` ... ` }}` starting at the current position.
        const OPEN: &[u8] = b"{{ shout:";
        const CLOSE: &[u8] = b" }}";
        if !line.starts_with(OPEN) {
            return None;
        }
        // Find the closing ` }}` after the open marker.
        let body_start = OPEN.len();
        let mut i = body_start;
        let close_at = loop {
            if i + CLOSE.len() > line.len() {
                return None;
            }
            if &line[i..i + CLOSE.len()] == CLOSE {
                break i;
            }
            i += 1;
        };
        // Capture TEXT as a source slice [body_start, close_at).
        let text: text::Value = seg
            .with_start(seg.start() + body_start)
            .with_stop(seg.start() + close_at)
            .into();
        // Consume the entire `{{ shout:TEXT }}` span. The inline dispatcher does
        // not pre-consume the trigger byte, so the parser advances the full match
        // (mirrors UserMention, which advances its complete `@name` length).
        let consumed = close_at + CLOSE.len();
        reader.advance(consumed);
        let node_ref = arena.new_node(ShoutInline::new(text));
        Some(node_ref)
    }
}

impl From<ShoutInlineParser> for AnyInlineParser {
    fn from(p: ShoutInlineParser) -> Self {
        AnyInlineParser::Extension(Box::new(p))
    }
}

struct ShoutInlineHtmlRenderer<W: TextWrite> {
    _phantom: core::marker::PhantomData<W>,
    writer: html::Writer,
}

impl<W: TextWrite> ShoutInlineHtmlRenderer<W> {
    fn with_options(html_opts: Options, _options: NoRendererOptions) -> Self {
        Self {
            _phantom: core::marker::PhantomData,
            writer: html::Writer::with_options(html_opts),
        }
    }
}

impl<W: TextWrite> RenderNode<W> for ShoutInlineHtmlRenderer<W> {
    fn render_node<'a>(
        &self,
        w: &mut W,
        source: &'a str,
        arena: &'a Arena,
        node_ref: NodeRef,
        entering: bool,
        _context: &mut renderer::Context,
    ) -> Result<WalkStatus> {
        if entering {
            let si = as_extension_data!(arena, node_ref, ShoutInline);
            self.writer.write(w, si.text.str(source))?;
            self.writer.write_safe_str(w, "!")?;
        }
        Ok(WalkStatus::Continue)
    }
}

impl<'cb, W> NodeRenderer<'cb, W> for ShoutInlineHtmlRenderer<W>
where
    W: TextWrite + 'cb,
{
    fn register_node_renderer_fn(self, nrr: &mut impl NodeRendererRegistry<'cb, W>) {
        nrr.register_node_renderer_fn(TypeId::of::<ShoutInline>(), BoxRenderNode::new(self));
    }
}

fn shout_inline_parser_extension() -> impl ParserExtension {
    parser_extension(|p| {
        // PRIORITY_EMPHASIS + 100: run after CommonMark emphasis so our `{{ ... }}` inline claims
        // the span (higher value = later); `{` is not a CommonMark inline trigger, so no real collision.
        p.add_inline_parser(
            ShoutInlineParser::new,
            NoParserOptions,
            PRIORITY_EMPHASIS + 100,
        );
    })
}

fn shout_inline_html_renderer_extension<'cb, W>() -> impl RendererExtension<'cb, W>
where
    W: TextWrite + 'cb,
{
    renderer_extension(|r| {
        r.add_node_renderer(ShoutInlineHtmlRenderer::with_options, NoRendererOptions);
    })
}

// (b) ShoutInline }}}

/// Builds the combined parser+renderer closure with both spike extensions wired in.
fn build_renderer() -> impl Fn(&mut String, &str) -> Result<()> {
    new_markdown_to_html(
        parser::Options::default(),
        html::Options::default(),
        upper_block_parser_extension().and(shout_inline_parser_extension()),
        upper_block_html_renderer_extension().and(shout_inline_html_renderer_extension()),
    )
}

#[test]
fn custom_block_directive_renders() {
    let render = build_renderer();
    let mut output = String::new();
    render(&mut output, "@upper hello world\n").unwrap();
    eprintln!("custom_block_directive_renders output: {output:?}");
    assert!(
        output.contains("HELLO WORLD"),
        "expected uppercased text in output, got: {output:?}"
    );
}

#[test]
fn inline_directive_renders() {
    let render = build_renderer();
    let mut output = String::new();
    render(&mut output, "value is {{ shout:done }}\n").unwrap();
    eprintln!("inline_directive_renders output: {output:?}");
    assert!(
        output.contains("done!"),
        "expected `done!` in output, got: {output:?}"
    );
}
