pub trait XmlToString {
    fn xml(&self) -> String;
}

macro_rules! xml_response {
    ($(#[$attrs: meta])* $name: ident$(<$($ty_lifetimes:lifetime)*>)? {
        $($(#[$field_attrs: meta])* $fw: ident $field_name: ident: $tye: ty,)*
    }) => {
        $(#[$attrs])*
        pub struct $name$(<$($ty_lifetimes)*>)? {
            #[serde(rename = "@xmlns:u")]
            pub xmlns: &'a str,
            $($(#[$field_attrs])*
            $fw $field_name: $tye,)*
        }

        impl$(<$($ty_lifetimes)*>)? Default for $name$(<$($ty_lifetimes)*>)? {
            fn default() -> Self {
                Self {
                    xmlns: "urn:schemas-upnp-org:service:AVTransport:1",
                    $($field_name: Default::default(),)*
                }
            }
        }

        impl$(<$($ty_lifetimes)*>)? super::XmlToString for $name$(<$($ty_lifetimes)*>)? {
            fn xml(&self) -> String {
                let mut buf = String::new();
                let ser = Serializer::with_root(&mut buf, Some(concat!("u:", stringify!($name)))).unwrap();
                self.serialize(ser).unwrap();
                buf
            }
        }
    };
}

pub mod avtransport;
pub mod renderingcontrol;
