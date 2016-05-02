Setup:

```bash
heroku apps:create kafka-rust-test
heroku addons:create heroku-kafka:beta-dev
```

Wait for cluster to show in `heroku config`...

```bash
git push heroku master
```
