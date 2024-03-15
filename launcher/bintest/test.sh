az ad signed-in-user show --query id -o tsv | az role assignment create \
    --role "Storage Blob Data Contributor" \
    --assignee @- \
    --scope "/subscriptions/8c7f35fe-b93f-48fb-aef1-0840c54a7422/resourcegroups/itxPerma/providers/Microsoft.Storage/storageAccounts/blobperma"
