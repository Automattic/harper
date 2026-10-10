# frozen_string_literal: true

require 'minitest/autorun'
require 'minitest/mock'
require 'tmpdir'
require_relative '../lib/desktop_release'

class DesktopReleaseTest < Minitest::Test
  class FakeGithub
    attr_accessor :release_list, :asset_list, :created_release, :create_error, :after_conflict,
                  :list_error, :upload_error, :ref_error
    attr_reader :calls

    def initialize
      @release_list = []
      @asset_list = []
      @calls = []
    end

    def releases(repository)
      @calls << [:releases, repository]
      raise list_error if list_error

      release_list
    end

    def ref(repository, reference)
      @calls << [:ref, repository, reference]
      raise ref_error if ref_error
    end

    def create_release(repository, tag, **options)
      @calls << [:create, repository, tag, options]
      if create_error
        @release_list = after_conflict if after_conflict
        raise create_error
      end
      created_release
    end

    def release_assets(url)
      @calls << [:assets, url]
      asset_list
    end

    def delete_release_asset(url)
      @calls << [:delete, url]
    end

    def upload_asset(url, path, **options)
      @calls << [:upload, url, path, options]
      raise upload_error if upload_error
    end
  end

  def setup
    @directory = Dir.mktmpdir('desktop-release')
    @installer = File.join(@directory, 'Harper_2.12.0_x64-setup.exe')
    File.write(@installer, 'test installer')
    @client = FakeGithub.new
    @draft = release(id: 123)
    @client.created_release = @draft
  end

  def teardown
    FileUtils.remove_entry(@directory)
  end

  def release(id:, tag: 'v2.12.0', draft: true)
    { id: id, tag_name: tag, draft: draft,
      url: "https://api.github.com/repos/example/harper/releases/#{id}",
      html_url: "https://github.com/example/harper/releases/#{id}" }
  end

  def upload(assets = [@installer])
    DesktopRelease.upload(client: @client, repository: 'example/harper', tag: 'v2.12.0', assets: assets)
  end

  def writes
    @client.calls.select { |call| %i[create delete upload].include?(call.first) }
  end

  def test_creates_a_draft_only_after_verifying_the_tag_exists
    assert_equal @draft[:html_url], upload
    assert_operator @client.calls.index { |call| call.first == :ref }, :<,
                    @client.calls.index { |call| call.first == :create }
    assert_includes @client.calls, [:ref, 'example/harper', 'tags/v2.12.0']
    assert_includes writes, [:create, 'example/harper', 'v2.12.0',
                            { name: 'v2.12.0', body: '', draft: true }]
  end

  def test_reuses_existing_draft_without_changing_its_notes_or_metadata
    @client.release_list = [@draft]
    upload
    assert_equal [:upload], writes.map(&:first)
    assert_equal @draft[:url], writes.first[1]
  end

  def test_replaces_matching_assets_and_preserves_unrelated_assets
    @client.release_list = [@draft]
    @client.asset_list = [
      { name: File.basename(@installer), url: 'old-windows' },
      { name: 'Harper_universal.dmg', url: 'macos' },
      { name: 'chrome-plugin.zip', url: 'chrome' }
    ]
    upload
    assert_equal %i[delete upload], writes.map(&:first)
    assert_equal [:delete, 'old-windows'], writes.first
  end

  def test_reuses_the_newest_draft_for_the_exact_tag
    newer = release(id: 124)
    @client.release_list = [release(id: 500, tag: 'v2.13.0'), @draft, newer]
    upload
    assert_equal newer[:url], writes.first[1]
  end

  def test_never_uses_a_release_for_another_tag
    @client.release_list = [release(id: 500, tag: 'v2.13.0', draft: false)]
    upload
    assert_equal :create, writes.first.first
    assert_equal 'v2.12.0', writes.first[2]
  end

  def test_rejects_a_published_release_even_when_a_newer_draft_exists
    @client.release_list = [release(id: 1, draft: false), @draft]
    assert_raises(RuntimeError) { upload }
    assert_empty writes
  end

  def test_retries_lookup_after_a_creation_conflict
    @client.create_error = Octokit::UnprocessableEntity.new
    @client.after_conflict = [@draft]
    assert_equal @draft[:html_url], upload
    assert_equal %i[create upload], writes.map(&:first)
  end

  def test_does_not_hide_an_unrelated_creation_error
    @client.create_error = Octokit::UnprocessableEntity.new
    assert_raises(Octokit::UnprocessableEntity) { upload }
    assert_equal [:create], writes.map(&:first)
  end

  def test_rejects_a_release_that_was_published_during_creation
    @client.create_error = Octokit::UnprocessableEntity.new
    @client.after_conflict = [release(id: 123, draft: false)]
    assert_raises(RuntimeError) { upload }
    assert_equal [:create], writes.map(&:first)
  end

  def test_does_not_create_a_release_after_a_lookup_failure
    @client.list_error = Octokit::Unauthorized.new
    assert_raises(Octokit::Unauthorized) { upload }
    assert_empty writes
  end

  def test_does_not_create_a_tag_when_the_expected_tag_is_missing
    @client.ref_error = Octokit::NotFound.new
    assert_raises(Octokit::NotFound) { upload }
    assert_empty writes
  end

  def test_upload_failure_is_not_reported_as_success
    @client.release_list = [@draft]
    @client.upload_error = Octokit::Unauthorized.new
    assert_raises(Octokit::Unauthorized) { upload }
  end

  def test_validates_all_files_before_modifying_a_release
    assert_raises(RuntimeError) { upload([@installer, File.join(@directory, 'missing.exe')]) }
    assert_empty @client.calls
  end

  def test_rejects_duplicate_filenames_before_modifying_a_release
    assert_raises(RuntimeError) { upload([@installer, @installer]) }
    assert_empty @client.calls
  end

  def test_rejects_an_empty_asset_list
    assert_raises(RuntimeError) { upload([]) }
    assert_empty @client.calls
  end

  def test_handles_macos_assets_with_the_same_upload_behavior
    @client.release_list = [@draft]
    paths = ['Harper.dmg', 'Harper.app.tar.gz', 'Harper.app.tar.gz.sig'].map do |name|
      path = File.join(@directory, name)
      File.write(path, 'test macOS artifact')
      path
    end
    upload(paths)
    assert_equal paths, writes.map { |call| call[2] }
  end

  def test_fastlane_resolves_script_paths_from_the_repository_root
    require 'fastlane'
    fastfile = Fastlane::FastFile.new(File.expand_path('../Fastfile', __dir__))
    lane = fastfile.runner.lanes[nil][:upload_desktop_release_assets]
    relative_path = 'harper-desktop/src-tauri/target/release/bundle/nsis/Harper-setup.exe'
    received = nil
    uploader = lambda do |**options|
      received = options
      'https://github.com/example/harper/releases/123'
    end

    Fastlane::Wpmreleasetoolkit::EnvManager.stub(:get_required_env!, 'test-token') do
      DesktopRelease.stub(:upload, uploader) do
        Dir.chdir(File.expand_path('..', __dir__)) do
          capture_io { lane.call(tag: 'v2.12.0', assets: [relative_path, @installer]) }
        end
      end
    end
    assert_equal [File.expand_path("../../#{relative_path}", __dir__), @installer], received[:assets]
    assert_equal 'v2.12.0', received[:tag]
  end
end
