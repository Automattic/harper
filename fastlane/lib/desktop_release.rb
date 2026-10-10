# frozen_string_literal: true

require 'octokit'

module DesktopRelease
  def self.upload(client:, repository:, tag:, assets:)
    raise 'At least one release asset is required' if assets.empty?
    raise 'Release assets must be existing files' unless assets.all? { |path| File.file?(path) }

    names = assets.map { |path| File.basename(path) }
    raise 'Release asset filenames must be unique' unless names.uniq == names

    release = find_draft(client, repository, tag)
    unless release
      # Never create a tag implicitly from the repository's default branch.
      client.ref(repository, "tags/#{tag}")
      begin
        release = client.create_release(repository, tag, name: tag, body: '', draft: true)
      rescue Octokit::UnprocessableEntity
        # Another release job may have created the draft since our lookup.
        release = find_draft(client, repository, tag)
        raise unless release
      end
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

  def self.find_draft(client, repository, tag)
    # The list endpoint includes drafts. Prefer the newest matching draft when
    # earlier retries left duplicates, but never ignore a published release.
    matches = client.releases(repository).select { |release| release[:tag_name] == tag }
    if matches.any? { |release| release[:draft] != true }
      raise 'Refusing to upload desktop assets to a published release'
    end
    matches.max_by { |release| release[:id] }
  end
  private_class_method :find_draft
end
