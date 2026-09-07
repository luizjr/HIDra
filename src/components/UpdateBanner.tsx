import type { UpdateState } from "../hooks";
import type { AvailableUpdate, UpdateSupport } from "../updater";
import { downloadUrl, releaseNotesUrl } from "../updater";

interface UpdateBannerProps {
  state: UpdateState;
  support: UpdateSupport;
  onInstall: (update: AvailableUpdate) => void;
  onDismiss: () => void;
}

/**
 * The strip that appears above the workspace when a release is out.
 *
 * It offers to install only where HIDra can actually do it. A build we did not
 * package — a source checkout, a distro's own package — gets a download link
 * instead of a button that would fail.
 */
export function UpdateBanner({
  state,
  support,
  onInstall,
  onDismiss,
}: UpdateBannerProps) {
  if (state.kind === "idle" || state.kind === "checking") return null;

  if (state.kind === "uptodate") {
    return (
      <div className="updatebar updatebar--quiet" role="status">
        <span className="updatebar__text">Você está na versão mais recente.</span>
        <button type="button" className="btn btn--sm" onClick={onDismiss}>
          Fechar
        </button>
      </div>
    );
  }

  if (state.kind === "error") {
    return (
      <div className="updatebar updatebar--error" role="alert">
        <span className="updatebar__text">
          Não foi possível atualizar: {state.message}
        </span>
        <a
          className="btn btn--sm"
          href={`${downloadUrl(support, "latest")}`}
          target="_blank"
          rel="noreferrer"
        >
          Baixar no site
        </a>
        <button type="button" className="btn btn--sm" onClick={onDismiss}>
          Fechar
        </button>
      </div>
    );
  }

  if (state.kind === "installing") {
    const pct = state.progress === null ? null : Math.round(state.progress * 100);
    return (
      <div className="updatebar" role="status" aria-live="polite">
        <span className="updatebar__text">
          {pct === null ? "Instalando atualização…" : `Baixando atualização… ${pct}%`}
        </span>
        <span className="updatebar__progress" aria-hidden="true">
          <span
            className={`updatebar__bar${pct === null ? " is-indeterminate" : ""}`}
            style={pct === null ? undefined : { width: `${pct}%` }}
          />
        </span>
      </div>
    );
  }

  const { update } = state;
  const canInstall = support.can_install;
  return (
    <div className="updatebar" role="status">
      <span className="updatebar__text">
        <strong>HIDra {update.version}</strong> disponível.
      </span>
      <a
        className="updatebar__link"
        href={releaseNotesUrl(update.version)}
        target="_blank"
        rel="noreferrer"
      >
        Ver novidades
      </a>
      {canInstall ? (
        <button
          type="button"
          className="btn btn--primary btn--sm"
          onClick={() => onInstall(update)}
          title={
            support.install === "deb" || support.install === "rpm"
              ? "Baixa o pacote assinado e instala pelo gerenciador do sistema"
              : undefined
          }
        >
          Atualizar agora
        </button>
      ) : (
        <a
          className="btn btn--primary btn--sm"
          href={downloadUrl(support, update.version)}
          target="_blank"
          rel="noreferrer"
        >
          Baixar
        </a>
      )}
      <button type="button" className="btn btn--sm" onClick={onDismiss}>
        Depois
      </button>
    </div>
  );
}
