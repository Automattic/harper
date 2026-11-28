# Configure the primary library bucket.
resource "aws_s3_bucket" "libraries" {
  // Enable versioning for safety.
  bucket = "example-libraries"

  /*
   * Apply standard environment tags.
   */
  tags = {
    Environment = "production"
  }
}
