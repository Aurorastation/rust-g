use ammonia::{Builder};
use url::Url;

byond_fn! { clean_html(data) {
    let clean = Builder::default()
        .clean(data)
        .to_string();
    Some(data)
} }

byond_fn! { clean_html_chat_trusted(data) {
    let allowed_classes = hashset!["admin", "admin_channel", "airadio", "alert", "alien", "alium", "attack", "bad", "cciaasay", "centradio", "changeling", "comradio", "cult", "danger", "deadsay", "delvahhi", "deptradio", "developer", "devsay", "disarm", "elevated", "emote", "engradio", "entradio", "everyone", "good", "howto", "in", "info", "interface", "log_message", "looc", "medradio", "mod_channel", "moderate", "moderator", "motd", "name", "newscaster", "notice", "ooc", "oocimg", "other", "out", "passive", "pm", "prefix", "psychic", "radio", "reflex_shoot", "rose", "rough", "say", "say_quote", "sciradio", "secradio", "siiktau", "skrell", "soghun", "soghun_alt", "solcom", "srvradio", "supradio", "syndradio", "tajaran", "tajaran_signlang", "text_tag", "vaurca", "vox", "warning", "yassa"];
    let clean = Builder::new()
        .tags(hashset!["a", "b", "br", "center", "cite", "code", "del", "em", "strong", "h1", "h2", "h3", "h4", "h5", "h6", "hr", "ins", "small", "span", "strike", "div"])
        .tag_attributes(hashmap![
            "a" => hashset!["href"]
        ];)
        .generic_attributes(hashset!["style"])
        .allowed_classes(hashmap![
            "span" => allowed_classes
        ])
        .clean(data)
        .to_string();
    Some(data)
} }

byond_fn! { clean_html_document(data) {
    let clean = Builder::default()
        .clean(data)
        .attribute_filter(html_attribute_filter)
        .to_string();
    Some(data)
} }

fn html_attribute_filter(element, attribute, value) {
    match (element, attribute) {
        ("a", "href") => url_sanitizer(value)
        _ => Some(value.into())
    }
}

fn url_sanitizer(value) {
    let url = Url:parse(value);
    match url.host_str(). {
        ""

    }
}