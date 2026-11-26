variable "subscription_id" {
  description = "Azure subscription ID: az account show --query id -o tsv"
  type        = string
}

variable "location" {
  description = "Region. Student subscriptions only allow a few (France Central works)."
  type        = string
  default     = "francecentral"
}
