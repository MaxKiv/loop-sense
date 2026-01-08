{
  inputs,
  config,
  username,
  hostname,
  pkgs,
  lib,
  sshPublicKeys,
  ...
}: {
  # Enable nginx service to serve the built React application
  services.nginx = {
    enable = true;

    virtualHosts."heart-os" = {
      serverName = "heart-os";
      default = true;  # Make this the default server for port 80
      root = "/var/www/heart_os";

      # SPA routing - serve index.html for all routes
      locations."/" = {
        extraConfig = ''
          try_files $uri $uri/ /index.html;
        '';
      };

      # Cache static assets
      locations."~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$" = {
        extraConfig = ''
          expires 1y;
          add_header Cache-Control "public, immutable";
        '';
      };
    };
  };
}

