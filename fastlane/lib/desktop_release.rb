# frozen_string_literal: true

require 'octokit'

# Coordinates macOS and Windows assets in a shared draft GitHub release.
module DesktopRelease
  # Create or reuse the tag's draft, replacing only assets with matching filenames.
  # @param tag [String] Existing Git tag.
  # @param assets [Array<String>] Files to upload, with unique basenames.
  # @return [String] Release URL.
  # @raise [RuntimeError] If assets are invalid, drafts are ambiguous, or the release is published.
  def self.upload(client:, repository:, tag:, assets:)
    raise 'At least one release asset is required' if assets.empty?
    raise 'Release assets must be existing files' unless assets.all? { |path| File.file?(path) }

    names = assets.map { |path| File.basename(path) }
    raise 'Release asset filenames must be unique' unless names.uniq == names

    release = find_draft(client, repository, tag)
    unless release
      # Never create a tag implicitly from the repository's default branch.
      client.ref(repository, "tags/#{tag}")
      release = client.create_release(repository, tag, name: tag, body: '', draft: true)
    end
    raise 'Refusing to upload desktop assets to a published release' unless release[:draft] == true

    existing_assets = client.release_assets(release[:url])
    assets.each do |path|
      existing_assets.select { |asset| asset[:name] == File.basename(path) }.each do |asset|
        client.delete_release_asset(asset[:url])
      end
      client.upload_asset(release[:url], path, content_type: 'application/octet-stream')
    end
    release[:html_url]
  end

  # Find the sole draft for the exact tag.
  # @return [Sawyer::Resource, nil] Matching draft, if one exists.
  # @raise [RuntimeError] If multiple drafts or a published release match.
  def self.find_draft(client, repository, tag)
    matches = client.releases(repository).select { |release| release[:tag_name] == tag }
    if matches.any? { |release| release[:draft] != true }
      raise 'Refusing to upload desktop assets to a published release'
    end
    if matches.length > 1
      ids = matches.map { |release| release[:id] }.sort.join(', ')
      raise "Multiple draft releases for #{tag}: #{ids}. Reconcile them before retrying."
    end
    matches.first
  end
  private_class_method :find_draft
end
