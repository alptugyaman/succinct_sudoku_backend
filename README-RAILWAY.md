# Railway Deployment Instructions

This document provides instructions for deploying the Succinct Sudoku Backend to Railway.

## Prerequisites

- A Railway account (https://railway.app/)
- Railway CLI installed (optional, for local development)

## Deployment Steps

### Option 1: Deploy via Railway Dashboard

1. Fork or clone this repository to your GitHub account.
2. Log in to your Railway account.
3. Click on "New Project" and select "Deploy from GitHub repo".
4. Select your forked/cloned repository.
5. Railway will automatically detect the Dockerfile and deploy your application.
6. Once deployed, you can access your application via the provided URL.

### Option 2: Deploy via Railway CLI

1. Install the Railway CLI:
   ```bash
   npm i -g @railway/cli
   ```

2. Login to your Railway account:
   ```bash
   railway login
   ```

3. Initialize a new project:
   ```bash
   railway init
   ```

4. Deploy the application:
   ```bash
   railway up
   ```

5. Open the deployed application:
   ```bash
   railway open
   ```

## Configuration

The application is configured to use the `PORT` environment variable provided by Railway. No additional configuration is required for basic functionality.

## Troubleshooting

If you encounter any issues during deployment:

1. Check the build logs in the Railway dashboard for errors.
2. Ensure that the SP1 prover is being built correctly during the Docker build process.
3. Verify that the application is listening on the correct port (0.0.0.0:$PORT).

## API Endpoints

Once deployed, you can access the API endpoints as described in the main README.md file, using your Railway-provided URL instead of localhost.

For example:
```
https://your-railway-app.railway.app/validate
https://your-railway-app.railway.app/verify
https://your-railway-app.railway.app/prove
``` 