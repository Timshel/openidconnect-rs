use std::fmt::{Debug, Formatter, Result as FormatterResult};
use std::marker::PhantomData;
use std::str;

use chrono::{DateTime, Utc};
use serde;
use serde::de::{Deserialize, DeserializeOwned, Deserializer, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use super::types::helpers::{seconds_to_utc, split_language_tag_key, utc_to_seconds};
use super::types::{LocalizedClaim, Seconds};
use super::{
    AddressCountry, AddressLocality, AddressPostalCode, AddressRegion, EndUserBirthday,
    EndUserEmail, EndUserFamilyName, EndUserGivenName, EndUserMiddleName, EndUserName,
    EndUserNickname, EndUserPhoneNumber, EndUserPictureUrl, EndUserProfileUrl, EndUserTimezone,
    EndUserUsername, EndUserWebsiteUrl, FormattedAddress, LanguageTag, StreetAddress,
    SubjectIdentifier,
};

pub trait AdditionalClaims: Debug + DeserializeOwned + Serialize + 'static {}

// In order to support serde flatten, this must be an empty struct rather than an empty
// tuple struct.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct EmptyAdditionalClaims {}
impl AdditionalClaims for EmptyAdditionalClaims {}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AddressClaim {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<FormattedAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_address: Option<StreetAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<AddressLocality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<AddressRegion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<AddressPostalCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<AddressCountry>,
}

pub trait GenderClaim: Clone + Debug + DeserializeOwned + Serialize + 'static {}

#[derive(Clone, Debug, PartialEq)]
pub struct StandardClaims<GC>
where
    GC: GenderClaim,
{
    pub(crate) sub: SubjectIdentifier,
    pub(crate) name: Option<LocalizedClaim<EndUserName>>,
    pub(crate) given_name: Option<LocalizedClaim<EndUserGivenName>>,
    pub(crate) family_name: Option<LocalizedClaim<EndUserFamilyName>>,
    pub(crate) middle_name: Option<LocalizedClaim<EndUserMiddleName>>,
    pub(crate) nickname: Option<LocalizedClaim<EndUserNickname>>,
    pub(crate) preferred_username: Option<EndUserUsername>,
    pub(crate) profile: Option<LocalizedClaim<EndUserProfileUrl>>,
    pub(crate) picture: Option<LocalizedClaim<EndUserPictureUrl>>,
    pub(crate) website: Option<LocalizedClaim<EndUserWebsiteUrl>>,
    pub(crate) email: Option<EndUserEmail>,
    pub(crate) email_verified: Option<bool>,
    pub(crate) gender: Option<GC>,
    pub(crate) birthday: Option<EndUserBirthday>,
    pub(crate) zoneinfo: Option<EndUserTimezone>,
    pub(crate) locale: Option<LanguageTag>,
    pub(crate) phone_number: Option<EndUserPhoneNumber>,
    pub(crate) phone_number_verified: Option<bool>,
    pub(crate) address: Option<AddressClaim>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
}
impl<GC> StandardClaims<GC>
where
    GC: GenderClaim,
{
    pub fn new(subject: SubjectIdentifier) -> Self {
        Self {
            sub: subject,
            name: None,
            given_name: None,
            family_name: None,
            middle_name: None,
            nickname: None,
            preferred_username: None,
            profile: None,
            picture: None,
            website: None,
            email: None,
            email_verified: None,
            gender: None,
            birthday: None,
            zoneinfo: None,
            locale: None,
            phone_number: None,
            phone_number_verified: None,
            address: None,
            updated_at: None,
        }
    }

    pub fn subject(&self) -> &SubjectIdentifier {
        &self.sub
    }
    pub fn set_subject(mut self, subject: SubjectIdentifier) -> Self {
        self.sub = subject;
        self
    }

    field_getters_setters![
        pub self [self] {
            set_name -> name[Option<LocalizedClaim<EndUserName>>],
            set_given_name -> given_name[Option<LocalizedClaim<EndUserGivenName>>],
            set_family_name ->
                family_name[Option<LocalizedClaim<EndUserFamilyName>>],
            set_middle_name ->
                middle_name[Option<LocalizedClaim<EndUserMiddleName>>],
            set_nickname -> nickname[Option<LocalizedClaim<EndUserNickname>>],
            set_preferred_username -> preferred_username[Option<EndUserUsername>],
            set_profile -> profile[Option<LocalizedClaim<EndUserProfileUrl>>],
            set_picture -> picture[Option<LocalizedClaim<EndUserPictureUrl>>],
            set_website -> website[Option<LocalizedClaim<EndUserWebsiteUrl>>],
            set_email -> email[Option<EndUserEmail>],
            set_email_verified -> email_verified[Option<bool>],
            set_gender -> gender[Option<GC>],
            set_birthday -> birthday[Option<EndUserBirthday>],
            set_zoneinfo -> zoneinfo[Option<EndUserTimezone>],
            set_locale -> locale[Option<LanguageTag>],
            set_phone_number -> phone_number[Option<EndUserPhoneNumber>],
            set_phone_number_verified -> phone_number_verified[Option<bool>],
            set_address -> address[Option<AddressClaim>],
            set_updated_at -> updated_at[Option<DateTime<Utc>>],
        }
    ];
}
impl<'de, GC> Deserialize<'de> for StandardClaims<GC>
where
    GC: GenderClaim,
{
    ///
    /// Special deserializer that supports [RFC 5646](https://tools.ietf.org/html/rfc5646) language
    /// tags associated with human-readable client metadata fields.
    ///
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ClaimsVisitor<GC: GenderClaim>(PhantomData<GC>);
        impl<'de, GC> Visitor<'de> for ClaimsVisitor<GC>
        where
            GC: GenderClaim,
        {
            type Value = StandardClaims<GC>;

            fn expecting(&self, formatter: &mut Formatter) -> FormatterResult {
                formatter.write_str("struct StandardClaims")
            }
            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                deserialize_fields! {
                    map {
                        [sub]
                        [LanguageTag(name)]
                        [LanguageTag(given_name)]
                        [LanguageTag(family_name)]
                        [LanguageTag(middle_name)]
                        [LanguageTag(nickname)]
                        [Option(preferred_username)]
                        [LanguageTag(profile)]
                        [LanguageTag(picture)]
                        [LanguageTag(website)]
                        [Option(email)]
                        [Option(email_verified)]
                        [Option(gender)]
                        [Option(birthday)]
                        [Option(zoneinfo)]
                        [Option(locale)]
                        [Option(phone_number)]
                        [Option(phone_number_verified)]
                        [Option(address)]
                        [Option(DateTime(Seconds(updated_at)))]
                    }
                }
            }
        }
        deserializer.deserialize_map(ClaimsVisitor(PhantomData))
    }
}
impl<GC> Serialize for StandardClaims<GC>
where
    GC: GenderClaim,
{
    #[allow(clippy::cognitive_complexity)]
    fn serialize<SE>(&self, serializer: SE) -> Result<SE::Ok, SE::Error>
    where
        SE: Serializer,
    {
        serialize_fields! {
            self -> serializer {
                [sub]
                [LanguageTag(name)]
                [LanguageTag(given_name)]
                [LanguageTag(family_name)]
                [LanguageTag(middle_name)]
                [LanguageTag(nickname)]
                [Option(preferred_username)]
                [LanguageTag(profile)]
                [LanguageTag(picture)]
                [LanguageTag(website)]
                [Option(email)]
                [Option(email_verified)]
                [Option(gender)]
                [Option(birthday)]
                [Option(zoneinfo)]
                [Option(locale)]
                [Option(phone_number)]
                [Option(phone_number_verified)]
                [Option(address)]
                [Option(DateTime(Seconds(updated_at)))]
            }
        }
    }
}
