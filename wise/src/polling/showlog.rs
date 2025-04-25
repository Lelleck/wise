/*
#[instrument(level = "debug", skip_all, fields(poller_id = ctx.id))]
pub async fn poll_showlog(
    mut manager: PollingManager,
    mut ctx: PollingContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: use the rx
    let mut waiter = PollWaiter::new(ctx.config.clone());
    let mut known_logs = vec![];

    loop {
        waiter.wait().await;
        let new_logs = ctx.pool.execute(|c| Box::pin(c.fetch_showlog(1))).await?;

        let untracked_logs = merge_logs(&mut known_logs, new_logs);
        for log in untracked_logs {
            handle_untracked_log(&log, &mut manager, &mut ctx).await;
        }
    }
}

/// Merge
fn merge_logs(old_logs: &mut Vec<LogLine>, mut new_logs: Vec<LogLine>) -> Vec<LogLine> {
    let untracked_logs = new_logs
        .iter()
        .filter(|new_log| !old_logs.contains(new_log))
        .map(|l| l.clone())
        .collect::<Vec<LogLine>>();

    old_logs.append(&mut new_logs);
    let cutoff = (Utc::now() - Duration::from_secs(60 * 2)).timestamp() as u64;
    old_logs.retain(|l| l.timestamp > cutoff);

    untracked_logs
}

async fn handle_untracked_log(
    log_line: &LogLine,
    manager: &mut PollingManager,
    ctx: &mut PollingContext,
) {
    ctx.tx.send_rcon(RconEvent::Log(log_line.clone()));
    match &log_line.kind {
        LogKind::Connect {
            player,
            has_connected,
        } => match has_connected {
            true => {
                debug!("Detected player {:?} connecting", player);
                manager.start_playerinfo_poller(player.clone()).await;
            }
            false => {
                debug!("Detected layer {:?} disconnecting", player);
                manager.stop_playerinfo_poller(player.clone()).await;
            }
        },
        LogKind::TeamSwitch {
            player,
            old_team,
            new_team,
        } => {
            debug!(
                "Detected player {:?} switching teams from {} to {}",
                player, old_team, new_team
            );
        }
        LogKind::Kill {
            killer,
            killer_faction,
            victim,
            victim_faction,
            is_teamkill,
            weapon,
        } => {
            let kill_type = if *is_teamkill { "team kill" } else { "kill" };
            debug!(
                "Detected killer {:?} on {} {} victim {:?} on {} with {}",
                killer, killer_faction, kill_type, victim, victim_faction, weapon
            );
        }
        LogKind::MatchStart { map } => debug!("Detected match start on {}", map),
        LogKind::MatchEnded {
            map,
            allied_score,
            axis_score,
        } => debug!(
            "Detected match end on {} with scores Allies: {} - Axis: {}",
            map, allied_score, axis_score
        ),
        _ => {}
    }
}

*/
