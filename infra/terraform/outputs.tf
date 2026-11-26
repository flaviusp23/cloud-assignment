output "server_name" {
  value = azurerm_postgresql_flexible_server.db.name
}

output "server_host" {
  value = azurerm_postgresql_flexible_server.db.fqdn
}

# Contains the password, so it is hidden in the plan output.
# Read it with: terraform output -raw db_url
output "db_url" {
  value     = "postgres://app:${random_password.db.result}@${azurerm_postgresql_flexible_server.db.fqdn}:5432/bookmarks?sslmode=require"
  sensitive = true
}
