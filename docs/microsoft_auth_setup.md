# Microsoft Authentication Setup Guide

Since we cannot use CAS, we will have to use Microsoft Entra instead.

## 1. Register the App (Azure Portal)

1.  **Sign In**: Go to [entra.microsoft.com](https://entra.microsoft.com/) and sign in with any Microsoft account.
2.  **Create App**: Navigate to **Identity > Applications > App registrations > New registration**.
3.  **Configure**:
    - **Name**: `cwdb-auth` (or similar).
    - **Supported account types**: Select **"Accounts in any organizational directory (Any Microsoft Entra ID tenant - Multitenant)"**. _This is crucial for allowing IIIT accounts to sign in._
    - **Redirect URI**: Select **Web** and enter: `http://localhost:3000/api/auth/callback` (Adjust port if different).
4.  **Register**: Click **Register**.
5.  **Get Client ID**: Copy the **Application (client) ID**.
6.  **Get Client Secret**:
    - Go to **Certificates & secrets > New client secret**.
    - Create one and copy the **Value** immediately (store it safely).

## 2. Configure Environment

Update your `.env` file with the credentials:

```ini
MS_CLIENT_ID=your_client_id_here
MS_CLIENT_SECRET=your_client_secret_here
MS_REDIRECT_URI=http://localhost:3000/api/auth/callback
```
