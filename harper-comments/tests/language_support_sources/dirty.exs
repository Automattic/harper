defmodule MyApp.MixProject do
  @moduledoc """
  Note: Spelling mistakes in the module doc and documentation are not picked up by Harper, as they are defined as quoted strings by the tree-sitter.

  This module defin the Mix project configuration for something
  """
  use Mix.Project

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  # Specifies project dependences
  defp deps do
    [
      # Example: {:jason, "~> 1.4"}
    ]
  end
end
