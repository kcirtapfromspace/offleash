# devenv.nix - Nix-based development environment
# Usage: Install devenv, then run `devenv shell` or use direnv with `devenv init`
#
# Install devenv: https://devenv.sh/getting-started/
{ pkgs, lib, config, inputs, ... }:

{
  # Environment name
  name = "dog-walker";

  # Environment variables
  env = {
    RUST_LOG = "debug,tower_http=debug,sqlx=warn";
    RUST_BACKTRACE = "1";
  };

  # Packages available in the environment
  packages = with pkgs; [
    # Rust toolchain (managed by devenv)
    cargo-watch
    cargo-nextest
    sqlx-cli

    # Database
    postgresql_16

    # Kubernetes/Container tools
    docker
    kubectl
    k3d
    tilt

    # Utilities
    jq
    httpie
    just  # Modern make alternative
  ];

  # Rust configuration
  languages.rust = {
    enable = true;
    channel = "stable";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };

  # PostgreSQL service
  services.postgres = {
    enable = true;
    package = pkgs.postgresql_16;
    initialDatabases = [{ name = "dog_walker"; }];
    initialScript = ''
      CREATE USER dogwalker WITH PASSWORD 'dogwalker_dev' SUPERUSER;
      GRANT ALL PRIVILEGES ON DATABASE dog_walker TO dogwalker;
    '';
    listen_addresses = "127.0.0.1";
    port = 5432;
  };

  # Environment hooks
  enterShell = ''
    echo ""
    echo "🐕 Dog Walker Development Environment"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Rust:     $(rustc --version)"
    echo "Cargo:    $(cargo --version)"
    echo "Postgres: $(psql --version)"
    echo ""
    echo "Database: postgres://dogwalker:dogwalker_dev@localhost:5432/dog_walker"
    echo ""
    echo "Commands:"
    echo "  devenv up           - Start PostgreSQL in background"
    echo "  cargo run --bin api - Run the API server"
    echo "  cargo watch -x run  - Run with live reload"
    echo "  just dev            - Start full dev stack"
    echo ""

    # Export DATABASE_URL after postgres is configured
    export DATABASE_URL="postgres://dogwalker:dogwalker_dev@localhost:5432/dog_walker"
  '';

  # Process management (alternative to devenv up)
  processes = {
    api.exec = "cargo watch -x 'run --bin api'";
  };

  # Pre-commit hooks (optional but recommended)
  pre-commit.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
  };

  # Scripts available in the environment
  scripts = {
    dev.exec = ''
      echo "Starting development server..."
      cargo watch -x 'run --bin api'
    '';

    test.exec = ''
      cargo nextest run
    '';

    migrate.exec = ''
      sqlx migrate run --source migrations
    '';

    db-reset.exec = ''
      sqlx database drop -y 2>/dev/null || true
      sqlx database create
      sqlx migrate run --source migrations
      echo "Database reset complete!"
    '';

    lint.exec = ''
      cargo clippy -- -D warnings
      cargo fmt -- --check
    '';
  };
}
