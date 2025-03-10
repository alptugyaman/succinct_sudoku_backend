# Sudoku Backend

## English

This project provides a backend service for validating Sudoku puzzles and generating zero-knowledge proofs (ZKP).

### Features

- Sudoku puzzle validation
- Zero-knowledge proof (ZKP) generation
- Real-time log tracking via WebSocket
- Proof status querying via REST API

### Development

#### Requirements

- Rust 1.70+
- Cargo

#### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/sudoku_backend.git
cd sudoku_backend

# Install dependencies and build the project
cargo build

# Run the application
cargo run
```

#### API Endpoints

- `GET /` - Health check
- `POST /api/validate` - Validate Sudoku puzzle
- `POST /api/verify` - Verify Sudoku solution
- `POST /api/zkp` - Generate zero-knowledge proof
- `POST /api/prove` - Start asynchronous proof generation
- `GET /api/proof/:job_id` - Track proof status via WebSocket
- `GET /api/proof-status/:job_id` - Query proof status via REST API
- `GET /api/logs/:job_id` - Track log messages via WebSocket

### Railway Deployment

Follow these steps to deploy this project to Railway:

1. Install the [Railway CLI](https://docs.railway.app/develop/cli):
   ```bash
   npm i -g @railway/cli
   ```

2. Login to your Railway account:
   ```bash
   railway login
   ```

3. Create a new project:
   ```bash
   railway init
   ```

4. Deploy the project:
   ```bash
   railway up
   ```

Alternatively, you can deploy via the Railway Dashboard:

1. Go to the [Railway Dashboard](https://railway.app/dashboard)
2. Select "New Project" > "GitHub Repo"
3. Choose this repository and click the "Deploy" button

#### Environment Variables

You can set the following environment variables in Railway:

- `PORT`: The port the application will listen on (default: 3000)
- `RUST_LOG`: Log level (default: info)

### License

This project is licensed under the [MIT License](LICENSE).

---

## Türkçe

Bu proje, Sudoku bulmacalarını doğrulamak ve zero-knowledge proof (ZKP) oluşturmak için bir backend servisi sağlar.

### Özellikler

- Sudoku bulmacalarını doğrulama
- Zero-knowledge proof (ZKP) oluşturma
- WebSocket ile gerçek zamanlı log takibi
- REST API ile proof durumu sorgulama

### Geliştirme

#### Gereksinimler

- Rust 1.70+
- Cargo

#### Kurulum

```bash
# Repoyu klonla
git clone https://github.com/yourusername/sudoku_backend.git
cd sudoku_backend

# Bağımlılıkları kur ve projeyi derle
cargo build

# Uygulamayı çalıştır
cargo run
```

#### API Endpoints

- `GET /` - Sağlık kontrolü
- `POST /api/validate` - Sudoku bulmacasını doğrula
- `POST /api/verify` - Sudoku çözümünü doğrula
- `POST /api/zkp` - Zero-knowledge proof oluştur
- `POST /api/prove` - Asenkron proof oluşturma işlemi başlat
- `GET /api/proof/:job_id` - WebSocket ile proof durumunu takip et
- `GET /api/proof-status/:job_id` - REST API ile proof durumunu sorgula
- `GET /api/logs/:job_id` - WebSocket ile log mesajlarını takip et

### Railway Deployment

Bu projeyi Railway'e deploy etmek için aşağıdaki adımları izleyin:

1. [Railway CLI](https://docs.railway.app/develop/cli) yükleyin:
   ```bash
   npm i -g @railway/cli
   ```

2. Railway hesabınıza giriş yapın:
   ```bash
   railway login
   ```

3. Yeni bir proje oluşturun:
   ```bash
   railway init
   ```

4. Projeyi deploy edin:
   ```bash
   railway up
   ```

Alternatif olarak, Railway Dashboard üzerinden de deploy edebilirsiniz:

1. [Railway Dashboard](https://railway.app/dashboard)'a gidin
2. "New Project" > "GitHub Repo" seçin
3. Bu repoyu seçin ve "Deploy" düğmesine tıklayın

#### Ortam Değişkenleri

Railway'de aşağıdaki ortam değişkenlerini ayarlayabilirsiniz:

- `PORT`: Uygulamanın dinleyeceği port (varsayılan: 3000)
- `RUST_LOG`: Log seviyesi (varsayılan: info)

### Lisans

Bu proje [MIT Lisansı](LICENSE) altında lisanslanmıştır. 