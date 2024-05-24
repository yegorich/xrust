//! Tests for XSLT defined generically

use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use std::collections::HashMap;
use url::Url;
use xrust::item::{Item, Node, Sequence, SequenceTrait};
use xrust::transform::context::{StaticContext, StaticContextBuilder};
use xrust::xdmerror::{Error, ErrorKind};
use xrust::xslt::from_document;

type F = Box<dyn FnMut(&str) -> Result<(), Error>>;

fn test_rig<N: Node, G, H, J>(
    src: impl AsRef<str>,
    style: impl AsRef<str>,
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<Sequence<N>, Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let srcdoc = parse_from_str(src.as_ref())?;
    let (styledoc, stylens) = parse_from_str_with_ns(style.as_ref())?;
    let mut stctxt = StaticContext::<F>::new();
    let mut ctxt = from_document(
        styledoc,
        stylens,
        None,
        |s| parse_from_str(s),
        |_| Ok(String::new()),
    )?;
    ctxt.context(vec![Item::Node(srcdoc.clone())], 0);
    ctxt.result_document(make_doc()?);
    ctxt.populate_key_values(&mut stctxt, srcdoc.clone())?;
    ctxt.evaluate(&mut stctxt)
}

fn test_msg_rig<N: Node, G, H, J>(
    src: impl AsRef<str>,
    style: impl AsRef<str>,
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(Sequence<N>, Vec<String>), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let srcdoc = parse_from_str(src.as_ref())?;
    let (styledoc, stylens) = parse_from_str_with_ns(style.as_ref())?;
    let mut msgs: Vec<String> = vec![];
    let mut stctxt = StaticContextBuilder::new()
        .message(|m| {
            msgs.push(String::from(m));
            Ok(())
        })
        .build();
    let mut ctxt = from_document(
        styledoc,
        stylens,
        None,
        |s| parse_from_str(s),
        |_| Ok(String::new()),
    )?;
    ctxt.context(vec![Item::Node(srcdoc.clone())], 0);
    ctxt.result_document(make_doc()?);
    let seq = ctxt.evaluate(&mut stctxt)?;
    Ok((seq, msgs))
}

pub fn generic_literal_text<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><Level1>one</Level1><Level1>two</Level1></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'>Found the document</xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_string() == "Found the document" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"Found the document\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_sys_prop<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><Level1>one</Level1><Level1>two</Level1></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:sequence select='system-property("xsl:version")'/>-<xsl:sequence select='system-property("xsl:product-version")'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_string()
        == format!(
            "0.9-{}.{}.{}",
            pkg_version_major!(),
            pkg_version_minor!(),
            pkg_version_patch!()
        )
    {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"{}\"",
                result.to_string(),
                format!(
                    "0.9-{}.{}.{}",
                    pkg_version_major!(),
                    pkg_version_minor!(),
                    pkg_version_patch!()
                )
            ),
        ))
    }
}

pub fn generic_value_of_1<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test>special &lt; less than</Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='child::*'><xsl:value-of select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_string() == "special &lt; less than" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"special &lt; less than\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_value_of_2<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test>special &lt; less than</Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='child::*'><xsl:value-of select='.' disable-output-escaping='yes'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_string() == "special < less than" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"special < less than\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_literal_element<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><Level1>one</Level1><Level1>two</Level1></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><answer>Made an element</answer></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "<answer>Made an element</answer>" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"<answer>Made an element</answer>\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_element<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><Level1>one</Level1><Level1>two</Level1></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:element name='answer{count(ancestor::*)}'>Made an element</xsl:element></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "<answer0>Made an element</answer0>" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"<answer0>Made an element</answer0>\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_apply_templates_1<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><Level1>one</Level1><Level1>two</Level1></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::*'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::text()'>found text</xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "found textfound text" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"found textfound text\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_apply_templates_2<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test>one<Level1/>two<Level1/>three<Level1/>four<Level1/></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'><xsl:apply-templates select='child::text()'/></xsl:template>
  <xsl:template match='child::Level1'>found Level1 element</xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "onetwothreefour" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"onetwothreefour\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_comment<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test>one<Level1/>two<Level1/>three<Level1/>four<Level1/></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Level1'><xsl:comment> this is a level 1 element </xsl:comment></xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "one<!-- this is a level 1 element -->two<!-- this is a level 1 element -->three<!-- this is a level 1 element -->four<!-- this is a level 1 element -->" {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Unknown,
                       format!("got result \"{}\", expected \"one<!-- this is a level 1 element -->two<!-- this is a level 1 element -->three<!-- this is a level 1 element -->four<!-- this is a level 1 element -->\"", result.to_string())))
    }
}

pub fn generic_pi<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test>one<Level1/>two<Level1/>three<Level1/>four<Level1/></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Level1'><xsl:processing-instruction name='piL1'>this is a level 1 element</xsl:processing-instruction></xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "one<?piL1 this is a level 1 element?>two<?piL1 this is a level 1 element?>three<?piL1 this is a level 1 element?>four<?piL1 this is a level 1 element?>" {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Unknown,
                       format!("got result \"{}\", expected \"one<?piL1 this is a level 1 element?>two<?piL1 this is a level 1 element?>three<?piL1 this is a level 1 element?>four<?piL1 this is a level 1 element?>\"", result.to_string())))
    }
}

pub fn generic_current<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test ref='one'><second name='foo'>I am foo</second><second name='one'>I am one</second></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'>
    <xsl:sequence select='child::*/child::second[attribute::name eq current()/attribute::ref]'/>
  </xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "<second name='one'>I am one</second>" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"<second name='one'>I am one</second>\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_key_1<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><one>blue</one><two>yellow</two><three>green</three><four>blue</four></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:key name='mykey' match='child::*' use='child::text()'/>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'>#blue = <xsl:sequence select='count(key("mykey", "blue"))'/></xsl:template>
  <xsl:template match='child::Test/child::*'>shouldn't see this</xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "#blue = 2" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"#blue = 2\"",
                result.to_string()
            ),
        ))
    }
}

// Although we have the source and stylesheet in files,
// they are inlined here to avoid dependency on I/O libraries
pub fn generic_issue_58<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        r#"<Example>
    <Title>XSLT in Rust</Title>
    <Paragraph>A simple document.</Paragraph>
</Example>
"#,
        r#"<xsl:stylesheet
	version="1.0"
	xmlns:dat="http://www.stormware.cz/schema/version_2/data.xsd"
	xmlns:int="http://www.stormware.cz/schema/version_2/intDoc.xsd"
	xmlns:xsl="http://www.w3.org/1999/XSL/Transform">

	<xsl:output method="xml" encoding="utf-8" indent="yes"/>

    <xsl:template match="child::Example">
        <dat:dataPack>
            <xsl:apply-templates/>
        </dat:dataPack>
    </xsl:template>
    <xsl:template match="child::Title">
        <int:head>
            <xsl:apply-templates/>
        </int:head>
    </xsl:template>
    <xsl:template match="child::Paragraph">
        <int:body>
            <xsl:apply-templates/>
        </int:body>
    </xsl:template>
</xsl:stylesheet>
"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml()
        == r#"<dat:dataPack xmlns:dat='http://www.stormware.cz/schema/version_2/data.xsd' xmlns:int='http://www.stormware.cz/schema/version_2/intDoc.xsd'>
    <int:head>XSLT in Rust</int:head>
    <int:body>A simple document.</int:body>
</dat:dataPack>"# {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!("not expected result"),
        ))
    }
}

pub fn generic_message_1<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let (result, msgs) = test_msg_rig(
        "<Test>one<Level1/>two<Level1/>three<Level1/>four<Level1/></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Level1'><xsl:message>here is a level 1 element</xsl:message><L/></xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_xml() == "one<L></L>two<L></L>three<L></L>four<L></L>" {
        if msgs.len() == 4 {
            if msgs[0] == "here is a level 1 element" {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Unknown,
                    format!(
                        "got message \"{}\", expected \"here is a level 1 element\"",
                        msgs[0]
                    ),
                ))
            }
        } else {
            Err(Error::new(
                ErrorKind::Unknown,
                format!("got {} messages, expected 4", msgs.len()),
            ))
        }
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"one<L></L>two<L></L>three<L></L>four<L></L>\"",
                result.to_string()
            ),
        ))
    }
}

pub fn generic_message_term<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    match test_msg_rig(
        "<Test>one<Level1/>two<Level1/>three<Level1/>four<Level1/></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Level1'><xsl:message terminate='yes'>here is a level 1 element</xsl:message><L/></xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    ) {
        Err(e) => {
            if e.kind == ErrorKind::Terminated
                && e.message == "here is a level 1 element"
                && e.code.unwrap().to_string() == "XTMM9000"
            {
                Ok(())
            } else {
                Err(Error::new(ErrorKind::Unknown, "incorrect error"))
            }
        }
        Ok(_) => Err(Error::new(
            ErrorKind::Unknown,
            "evaluation succeeded when it should have failed",
        )),
    }
}
pub fn generic_callable_named_1<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><one>blue</one><two>yellow</two><three>green</three><four>blue</four></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'>
    <xsl:call-template name='my_template'>
      <xsl:with-param name='my_param' select='count(child::*)'/>
    </xsl:call-template>
  </xsl:template>
  <xsl:template name='my_template'>
    <xsl:param name='my_param'>default value</xsl:param>
    <xsl:text>There are </xsl:text>
    <xsl:sequence select='$my_param'/>
    <xsl:text> child elements</xsl:text>
  </xsl:template>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_string() == "There are 4 child elements" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"There are 4 child elements\"",
                result.to_string()
            ),
        ))
    }
}
pub fn generic_callable_posn_1<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let result = test_rig(
        "<Test><one>blue</one><two>yellow</two><three>green</three><four>blue</four></Test>",
        r#"<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform' xmlns:eg='http://example.org/'>
  <xsl:template match='/'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Test'>
    <xsl:sequence select='eg:my_func(count(child::*))'/>
  </xsl:template>
  <xsl:function name='eg:my_func'>
    <xsl:param name='my_param'/>
    <xsl:text>There are </xsl:text>
    <xsl:sequence select='$my_param'/>
    <xsl:text> child elements</xsl:text>
  </xsl:function>
</xsl:stylesheet>"#,
        parse_from_str,
        parse_from_str_with_ns,
        make_doc,
    )?;
    if result.to_string() == "There are 4 child elements" {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::Unknown,
            format!(
                "got result \"{}\", expected \"There are 4 child elements\"",
                result.to_string()
            ),
        ))
    }
}
pub fn generic_include<N: Node, G, H, J>(
    parse_from_str: G,
    parse_from_str_with_ns: J,
    make_doc: H,
) -> Result<(), Error>
where
    G: Fn(&str) -> Result<N, Error>,
    H: Fn() -> Result<N, Error>,
    J: Fn(&str) -> Result<(N, Vec<HashMap<String, String>>), Error>,
{
    let srcdoc =
        parse_from_str("<Test>one<Level1/>two<Level2/>three<Level3/>four<Level4/></Test>")?;
    let (styledoc, stylens) = parse_from_str_with_ns(
        "<xsl:stylesheet xmlns:xsl='http://www.w3.org/1999/XSL/Transform'>
  <xsl:include href='included.xsl'/>
  <xsl:template match='child::Test'><xsl:apply-templates/></xsl:template>
  <xsl:template match='child::Level1'>found Level1 element</xsl:template>
  <xsl:template match='child::text()'><xsl:sequence select='.'/></xsl:template>
</xsl:stylesheet>",
    )?;
    let pwd = std::env::current_dir().expect("unable to get current directory");
    let pwds = pwd
        .into_os_string()
        .into_string()
        .expect("unable to convert pwd");
    let mut stctxt = StaticContext::<F>::new();
    let mut ctxt = from_document(
        styledoc,
        stylens,
        Some(
            Url::parse(format!("file://{}/tests/xsl/including.xsl", pwds.as_str()).as_str())
                .expect("unable to parse URL"),
        ),
        |s| parse_from_str(s),
        |_| Ok(String::new()),
    )?;
    ctxt.context(vec![Item::Node(srcdoc.clone())], 0);
    ctxt.result_document(make_doc()?);
    let result = ctxt.evaluate(&mut stctxt)?;
    if result.to_string()
        == "onefound Level1 elementtwofound Level2 elementthreefound Level3 elementfour"
    {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Unknown, format!("got result \"{}\", expected \"onefound Level1 elementtwofound Level2 elementthreefound Level3 elementfour\"", result.to_string())))
    }
}
