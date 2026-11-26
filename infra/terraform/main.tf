# The managed resource: an Azure PostgreSQL server.
#   export TF_VAR_subscription_id=$(az account show --query id -o tsv)
#   terraform init && terraform apply        (terraform destroy when done)

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

variable "subscription_id" {
  type = string
}

# France Central refuses the B1ms size for this student subscription.
variable "location" {
  type    = string
  default = "polandcentral"
}

provider "azurerm" {
  features {}
  subscription_id = var.subscription_id
}

resource "azurerm_resource_group" "db" {
  name     = "rg-bookmarks"
  location = var.location
}

# Generated here, never written in code. It lives in the state file (not committed).
resource "random_password" "db" {
  length  = 24
  special = false
}

# Server names are global: a random suffix avoids clashes.
resource "random_string" "suffix" {
  length  = 6
  upper   = false
  special = false
}

# The cheapest size: burstable B1ms, 32 GiB.
resource "azurerm_postgresql_flexible_server" "db" {
  name                          = "psql-bookmarks-${random_string.suffix.result}"
  resource_group_name           = azurerm_resource_group.db.name
  location                      = azurerm_resource_group.db.location
  version                       = "16"
  sku_name                      = "B_Standard_B1ms"
  storage_mb                    = 32768
  zone                          = "1" # Azure picks one if unset; stating it keeps `plan` clean
  administrator_login           = "app"
  administrator_password        = random_password.db.result
  public_network_access_enabled = true
}

resource "azurerm_postgresql_flexible_server_database" "bookmarks" {
  name      = "bookmarks"
  server_id = azurerm_postgresql_flexible_server.db.id
}

# 0.0.0.0 is Azure's special rule for "allow other Azure services", such as the Container App.
resource "azurerm_postgresql_flexible_server_firewall_rule" "azure_services" {
  name             = "allow-azure-services"
  server_id        = azurerm_postgresql_flexible_server.db.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "0.0.0.0"
}

# Contains the password. Read it with: terraform output -raw db_url
output "db_url" {
  value     = "postgres://app:${random_password.db.result}@${azurerm_postgresql_flexible_server.db.fqdn}:5432/bookmarks?sslmode=require"
  sensitive = true
}
