variable "subscription_id" {
  description = "Azure subscription ID: az account show --query id -o tsv"
  type        = string
}

variable "location" {
  description = "Region. Student subscriptions only allow a few, and B1ms is only available in some (Poland Central works, France Central does not)."
  type        = string
  default     = "polandcentral"
}
