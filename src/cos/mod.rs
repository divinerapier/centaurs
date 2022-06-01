use std::str::FromStr;

use aws_endpoint::{partition::endpoint, CredentialScope};
use aws_sdk_s3::{Credentials, Region as AWSRegion};

pub enum Region {
    Beijing,
}

pub struct UnknownRegionError(String);

impl FromStr for Region {
    type Err = UnknownRegionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ap-beijing" => Ok(Region::Beijing),
            _ => Err(UnknownRegionError(s.to_string())),
        }
    }
}

impl Region {
    pub fn value(&self) -> &'static str {
        match self {
            Region::Beijing => "ap-beijing",
        }
    }
}

pub struct Client {
    pub client: aws_sdk_s3::Client,
    pub bucket: String,
    pub region: &'static str,
}

#[derive(Default)]
pub struct ClientBuilder {
    access_key: Option<String>,
    secret_key: Option<String>,
    bucket: Option<String>,
    region: Option<Region>,
}

impl ClientBuilder {
    pub fn access_key(mut self, ak: String) -> Self {
        self.access_key = Some(ak.trim().to_string());
        self
    }
    pub fn secret_key(mut self, sk: String) -> Self {
        self.secret_key = Some(sk.trim().to_string());
        self
    }
    pub fn bucket(mut self, bucket: String) -> Self {
        self.bucket = Some(bucket.trim().to_string());
        self
    }
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }
    pub fn build(self) -> Client {
        let credentials = Credentials::new(
            self.access_key.unwrap(),
            self.secret_key.unwrap(),
            None,
            None,
            "cos",
        );

        let endpoint = endpoint::Metadata {
            // Important: Replace the $accountNumber below with your AWS account number,
            // and $accessPoint with the name of your S3 Object Lambda access point.
            // For example, "my-access-point-123123123123.s3-object-lambda.{region}.amazonaws.com".
            // If you're using a FIPs region, add `-fips` after `s3-object-lambda`.
            uri_template: "cos.{region}.myqcloud.com",
            protocol: endpoint::Protocol::Https,
            signature_versions: endpoint::SignatureVersion::V4,
            // Important: The following overrides the credential scope so that request signing works.
            credential_scope: CredentialScope::builder()
                .service("s3-object-lambda")
                .build(),
        };

        let region = self.region.unwrap().value();

        let config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .endpoint_resolver(endpoint)
            .region(AWSRegion::new(region))
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        Client {
            client,
            bucket: self.bucket.unwrap(),
            region,
        }
    }
}

impl Client {
    pub fn builder() -> ClientBuilder {
        Default::default()
    }
}
