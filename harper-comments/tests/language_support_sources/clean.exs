defmodule MyApp.MixProject do
  use Mix.Project

  def project do
    [
      # Your app's atom name
      app: :my_app,
      # Version string
      version: "0.1.0",
      # Required Elixir version
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      # Dependencies (see below)
      deps: deps()
    ]
  end

  # When building an elixir project, the application function is used to define
  # the application configuration. It specifies which applications should be
  # started automatically when your application starts.

  def application do
    [
      # Apps to start automatically (e.g., :logger)
      extra_applications: [:logger]
    ]
  end

  defp some_function do
    # Some function implementation
    :ok
  end
end
