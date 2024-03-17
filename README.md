# s3presignedkey_rust_lambda
This cargo lambda function is an extension of the [ImageStorage](https://github.com/matthold86/ImageStorage) repository and contains the Rust lambda function for retrieving a presigned key from an AWS S3 bucket. This lambda function is part of a larger image processing pipeline that allows a user to upload and retrieve an image from an S3 bucket via a web application interface.

### AWS X-Ray

AWS X-Ray was added to this lambda function for improved troubleshooting and performance monitoring of the image processing pipeline. X-Ray tracing allows requests to be tracked between lambda functions in the pipeline showing how the requests propagate across the different microservices.
