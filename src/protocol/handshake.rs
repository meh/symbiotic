// This file is generated. Do not edit

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[deriving(Clone,Default)]
pub struct Version {
    major: ::std::option::Option<u32>,
    minor: ::std::option::Option<u32>,
    patch: ::std::option::Option<u32>,
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Version {
    pub fn new() -> Version {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Version {
        static mut instance: ::protobuf::lazy::Lazy<Version> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Version,
        };
        unsafe {
            instance.get(|| {
                Version {
                    major: ::std::option::None,
                    minor: ::std::option::None,
                    patch: ::std::option::None,
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required uint32 major = 1;

    pub fn clear_major(&mut self) {
        self.major = ::std::option::None;
    }

    pub fn has_major(&self) -> bool {
        self.major.is_some()
    }

    // Param is passed by value, moved
    pub fn set_major(&mut self, v: u32) {
        self.major = ::std::option::Some(v);
    }

    pub fn get_major<'a>(&self) -> u32 {
        self.major.unwrap_or(0)
    }

    // required uint32 minor = 2;

    pub fn clear_minor(&mut self) {
        self.minor = ::std::option::None;
    }

    pub fn has_minor(&self) -> bool {
        self.minor.is_some()
    }

    // Param is passed by value, moved
    pub fn set_minor(&mut self, v: u32) {
        self.minor = ::std::option::Some(v);
    }

    pub fn get_minor<'a>(&self) -> u32 {
        self.minor.unwrap_or(0)
    }

    // required uint32 patch = 3;

    pub fn clear_patch(&mut self) {
        self.patch = ::std::option::None;
    }

    pub fn has_patch(&self) -> bool {
        self.patch.is_some()
    }

    // Param is passed by value, moved
    pub fn set_patch(&mut self, v: u32) {
        self.patch = ::std::option::Some(v);
    }

    pub fn get_patch<'a>(&self) -> u32 {
        self.patch.unwrap_or(0)
    }
}

impl ::protobuf::Message for Version {
    fn new() -> Version {
        Version::new()
    }

    fn is_initialized(&self) -> bool {
        if self.major.is_none() {
            return false;
        };
        if self.minor.is_none() {
            return false;
        };
        if self.patch.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_uint32());
                    self.major = ::std::option::Some(tmp);
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_uint32());
                    self.minor = ::std::option::Some(tmp);
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = try!(is.read_uint32());
                    self.patch = ::std::option::Some(tmp);
                },
                _ => {
                    let unknown = try!(is.read_unknown(wire_type));
                    self.mut_unknown_fields().add_value(field_number, unknown);
                },
            };
        }
        ::std::result::Ok(())
    }

    // Compute sizes of nested messages
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.major.iter() {
            my_size += ::protobuf::rt::value_size(1, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        for value in self.minor.iter() {
            my_size += ::protobuf::rt::value_size(2, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        for value in self.patch.iter() {
            my_size += ::protobuf::rt::value_size(3, *value, ::protobuf::wire_format::WireTypeVarint);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.major {
            try!(os.write_uint32(1, v));
        };
        if let Some(v) = self.minor {
            try!(os.write_uint32(2, v));
        };
        if let Some(v) = self.patch {
            try!(os.write_uint32(3, v));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s ::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    #[allow(unused_unsafe,unused_mut)]
    fn descriptor_static(_: ::std::option::Option<Version>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_u32_accessor(
                    "major",
                    Version::has_major,
                    Version::get_major,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_u32_accessor(
                    "minor",
                    Version::has_minor,
                    Version::get_minor,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_u32_accessor(
                    "patch",
                    Version::has_patch,
                    Version::get_patch,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Version>(
                    "Version",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn type_id(&self) -> ::std::intrinsics::TypeId {
        ::std::intrinsics::TypeId::of::<Version>()
    }
}

impl ::protobuf::Clear for Version {
    fn clear(&mut self) {
        self.clear_major();
        self.clear_minor();
        self.clear_patch();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.major == other.major &&
        self.minor == other.minor &&
        self.patch == other.patch &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Show for Version {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        self.fmt_impl(f)
    }
}

#[deriving(Clone,Default)]
pub struct Identity {
    name: ::protobuf::SingularField<::std::string::String>,
    version: ::protobuf::SingularPtrField<Version>,
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::std::cell::Cell<u32>,
}

impl Identity {
    pub fn new() -> Identity {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Identity {
        static mut instance: ::protobuf::lazy::Lazy<Identity> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Identity,
        };
        unsafe {
            instance.get(|| {
                Identity {
                    name: ::protobuf::SingularField::none(),
                    version: ::protobuf::SingularPtrField::none(),
                    unknown_fields: ::protobuf::UnknownFields::new(),
                    cached_size: ::std::cell::Cell::new(0),
                }
            })
        }
    }

    // required string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = ::protobuf::SingularField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name<'a>(&'a mut self) -> &'a mut ::std::string::String {
        if self.name.is_none() {
            self.name.set_default();
        };
        self.name.as_mut().unwrap()
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        self.name.take().unwrap_or_else(|| ::std::string::String::new())
    }

    pub fn get_name<'a>(&'a self) -> &'a str {
        match self.name.as_ref() {
            Some(v) => v.as_slice(),
            None => "",
        }
    }

    // required .symbiotic.protocol.handshake.Version version = 2;

    pub fn clear_version(&mut self) {
        self.version.clear();
    }

    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    // Param is passed by value, moved
    pub fn set_version(&mut self, v: Version) {
        self.version = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_version<'a>(&'a mut self) -> &'a mut Version {
        if self.version.is_none() {
            self.version.set_default();
        };
        self.version.as_mut().unwrap()
    }

    // Take field
    pub fn take_version(&mut self) -> Version {
        self.version.take().unwrap_or_else(|| Version::new())
    }

    pub fn get_version<'a>(&'a self) -> &'a Version {
        self.version.as_ref().unwrap_or_else(|| Version::default_instance())
    }
}

impl ::protobuf::Message for Identity {
    fn new() -> Identity {
        Identity::new()
    }

    fn is_initialized(&self) -> bool {
        if self.name.is_none() {
            return false;
        };
        if self.version.is_none() {
            return false;
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !try!(is.eof()) {
            let (field_number, wire_type) = try!(is.read_tag_unpack());
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.name.set_default();
                    try!(is.read_string_into(tmp))
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeLengthDelimited {
                        return ::std::result::Err(::protobuf::ProtobufError::WireError("unexpected wire type".to_string()));
                    };
                    let tmp = self.version.set_default();
                    try!(is.merge_message(tmp))
                },
                _ => {
                    let unknown = try!(is.read_unknown(wire_type));
                    self.mut_unknown_fields().add_value(field_number, unknown);
                },
            };
        }
        ::std::result::Ok(())
    }

    // Compute sizes of nested messages
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in self.name.iter() {
            my_size += ::protobuf::rt::string_size(1, value.as_slice());
        };
        for value in self.version.iter() {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(v) = self.name.as_ref() {
            try!(os.write_string(1, v.as_slice()));
        };
        if let Some(v) = self.version.as_ref() {
            try!(os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited));
            try!(os.write_raw_varint32(v.get_cached_size()));
            try!(v.write_to_with_cached_sizes(os));
        };
        try!(os.write_unknown_fields(self.get_unknown_fields()));
        ::std::result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields<'s>(&'s self) -> &'s ::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields<'s>(&'s mut self) -> &'s mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    #[allow(unused_unsafe,unused_mut)]
    fn descriptor_static(_: ::std::option::Option<Identity>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_string_accessor(
                    "name",
                    Identity::has_name,
                    Identity::get_name,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_message_accessor(
                    "version",
                    Identity::has_version,
                    Identity::get_version,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Identity>(
                    "Identity",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn type_id(&self) -> ::std::intrinsics::TypeId {
        ::std::intrinsics::TypeId::of::<Identity>()
    }
}

impl ::protobuf::Clear for Identity {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_version();
        self.unknown_fields.clear();
    }
}

impl ::std::cmp::PartialEq for Identity {
    fn eq(&self, other: &Identity) -> bool {
        self.name == other.name &&
        self.version == other.version &&
        self.unknown_fields == other.unknown_fields
    }
}

impl ::std::fmt::Show for Identity {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        self.fmt_impl(f)
    }
}

static file_descriptor_proto_data: &'static [u8] = &[
    0x0a, 0x0f, 0x68, 0x61, 0x6e, 0x64, 0x73, 0x68, 0x61, 0x6b, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74,
    0x6f, 0x12, 0x1c, 0x73, 0x79, 0x6d, 0x62, 0x69, 0x6f, 0x74, 0x69, 0x63, 0x2e, 0x70, 0x72, 0x6f,
    0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x2e, 0x68, 0x61, 0x6e, 0x64, 0x73, 0x68, 0x61, 0x6b, 0x65, 0x22,
    0x36, 0x0a, 0x07, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x12, 0x0d, 0x0a, 0x05, 0x6d, 0x61,
    0x6a, 0x6f, 0x72, 0x18, 0x01, 0x20, 0x02, 0x28, 0x0d, 0x12, 0x0d, 0x0a, 0x05, 0x6d, 0x69, 0x6e,
    0x6f, 0x72, 0x18, 0x02, 0x20, 0x02, 0x28, 0x0d, 0x12, 0x0d, 0x0a, 0x05, 0x70, 0x61, 0x74, 0x63,
    0x68, 0x18, 0x03, 0x20, 0x02, 0x28, 0x0d, 0x22, 0x50, 0x0a, 0x08, 0x49, 0x64, 0x65, 0x6e, 0x74,
    0x69, 0x74, 0x79, 0x12, 0x0c, 0x0a, 0x04, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01, 0x20, 0x02, 0x28,
    0x09, 0x12, 0x36, 0x0a, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x18, 0x02, 0x20, 0x02,
    0x28, 0x0b, 0x32, 0x25, 0x2e, 0x73, 0x79, 0x6d, 0x62, 0x69, 0x6f, 0x74, 0x69, 0x63, 0x2e, 0x70,
    0x72, 0x6f, 0x74, 0x6f, 0x63, 0x6f, 0x6c, 0x2e, 0x68, 0x61, 0x6e, 0x64, 0x73, 0x68, 0x61, 0x6b,
    0x65, 0x2e, 0x56, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x4a, 0x9b, 0x03, 0x0a, 0x06, 0x12, 0x04,
    0x00, 0x00, 0x0b, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x00, 0x08, 0x24, 0x0a, 0x0a,
    0x0a, 0x02, 0x04, 0x00, 0x12, 0x04, 0x02, 0x00, 0x06, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x00,
    0x01, 0x12, 0x03, 0x02, 0x08, 0x0f, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x00, 0x12, 0x03,
    0x03, 0x08, 0x22, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x04, 0x12, 0x03, 0x03, 0x08,
    0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05, 0x12, 0x03, 0x03, 0x11, 0x17, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03, 0x03, 0x18, 0x1d, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x03, 0x20, 0x21, 0x0a, 0x0b, 0x0a, 0x04, 0x04,
    0x00, 0x02, 0x01, 0x12, 0x03, 0x04, 0x08, 0x22, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01,
    0x04, 0x12, 0x03, 0x04, 0x08, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12,
    0x03, 0x04, 0x11, 0x17, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x04,
    0x18, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x03, 0x12, 0x03, 0x04, 0x20, 0x21,
    0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x02, 0x12, 0x03, 0x05, 0x08, 0x22, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x00, 0x02, 0x02, 0x04, 0x12, 0x03, 0x05, 0x08, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x00, 0x02, 0x02, 0x05, 0x12, 0x03, 0x05, 0x11, 0x17, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02,
    0x02, 0x01, 0x12, 0x03, 0x05, 0x18, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x02, 0x03,
    0x12, 0x03, 0x05, 0x20, 0x21, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x01, 0x12, 0x04, 0x08, 0x00, 0x0b,
    0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x01, 0x01, 0x12, 0x03, 0x08, 0x08, 0x10, 0x0a, 0x0b, 0x0a,
    0x04, 0x04, 0x01, 0x02, 0x00, 0x12, 0x03, 0x09, 0x08, 0x25, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01,
    0x02, 0x00, 0x04, 0x12, 0x03, 0x09, 0x08, 0x10, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00,
    0x05, 0x12, 0x03, 0x09, 0x11, 0x17, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12,
    0x03, 0x09, 0x19, 0x1d, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x03, 0x12, 0x03, 0x09,
    0x23, 0x24, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x02, 0x01, 0x12, 0x03, 0x0a, 0x08, 0x25, 0x0a,
    0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x01, 0x04, 0x12, 0x03, 0x0a, 0x08, 0x10, 0x0a, 0x0c, 0x0a,
    0x05, 0x04, 0x01, 0x02, 0x01, 0x06, 0x12, 0x03, 0x0a, 0x11, 0x18, 0x0a, 0x0c, 0x0a, 0x05, 0x04,
    0x01, 0x02, 0x01, 0x01, 0x12, 0x03, 0x0a, 0x19, 0x20, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02,
    0x01, 0x03, 0x12, 0x03, 0x0a, 0x23, 0x24,
];

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
