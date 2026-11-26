# The managed resource for the app: an Azure Database for PostgreSQL flexible server.
#   terraform init && terraform plan && terraform apply
#   terraform destroy      <- when you are done: it deletes everything below

terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 5.15"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.9"
    }
  }
}

provider "azurerm" {
  features {}
  subscription_id = var.subscription_id
}

resource "azurerm_resource_group" "db" {
  name     = "rg-bookmarks"
  location = var.location
}

# The password is generated here and never written in code. It lives in the
# Terraform state (never commit it) and in a Container App secret.
resource "random_password" "db" {
  length  = 24
  special = false
}

# Server names are global, so a random suffix avoids clashes.
resource "random_string" "suffix" {
  length  = 6
  upper   = false
  special = false
}

# The smallest, cheapest size: burstable B1ms, 32 GiB, no high availability.
resource "azurerm_postgresql_flexible_server" "db" {
  name                          = "psql-bookmarks-${random_string.suffix.result}"
  resource_group_name           = azurerm_resource_group.db.name
  location                      = azurerm_resource_group.db.location
  version                       = "16"
  sku_name                      = "B_Standard_B1ms"
  storage_mb                    = 32768
  administrator_login           = "app"
  administrator_password        = random_password.db.result
  public_network_access_enabled = true
}

resource "azurerm_postgresql_flexible_server_database" "bookmarks" {
  name      = "bookmarks"
  server_id = azurerm_postgresql_flexible_server.db.id
  charset   = "UTF8"
  collation = "en_US.utf8"
}

# 0.0.0.0 - 0.0.0.0 is Azure's special rule: "allow other Azure services",
# which is what lets the Container App reach the database.
resource "azurerm_postgresql_flexible_server_firewall_rule" "azure_services" {
  name             = "allow-azure-services"
  server_id        = azurerm_postgresql_flexible_server.db.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "0.0.0.0"
}
