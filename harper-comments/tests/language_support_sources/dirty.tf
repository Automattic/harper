# This this is an bad Terraform resource.
resource "aws_s3_bucket" "get_librraries" {
  // Store the librrary objects here.
  bucket = "example-librraries"

  /*
   * Apply an tag for tracking.
   */
  tags = {
    Environment = "production"
  }
}
