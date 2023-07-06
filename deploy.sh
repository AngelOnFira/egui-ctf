nomad job restart \
    -address=http://localhost:4646 \
    -group ctf-backend \
    -task actix-backend \
    ctf-dashboard

nomad job restart \
    -address=http://localhost:4646 \
    -group ctf-discord-bot \
    -task serenity-bot \
    ctf-dashboard
