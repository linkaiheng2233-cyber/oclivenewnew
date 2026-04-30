<script setup lang="ts">
import { Teleport } from "vue";
import { useI18n } from "vue-i18n";
import { usePluginCommunityMarketPane } from "../../composables/usePluginCommunityMarketPane";

const m = usePluginCommunityMarketPane();
const { t } = useI18n();
</script>

<template>
  <div class="pm2-market-wrap">
    <Teleport to="body">
      <div
        v-if="m.preflightVisible.value"
        class="pm2-modal-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="String(t('pluginMarketV2.preflight.dialogLabel'))"
        @click.self="m.onPreflightCancel"
      >
        <div class="pm2-modal" @click.stop>
          <div class="pm2-modal-h">{{ m.preflightTitle }}</div>
          <p class="pm2-hint">{{ t("pluginMarketV2.preflight.hint") }}</p>
          <ul class="pm2-preflight-list">
            <li v-for="(x, idx) in m.preflightLines" :key="`pl-${idx}`" class="pm2-preflight-li">
              <span style="white-space: pre-wrap">{{ x }}</span>
            </li>
          </ul>
          <div class="pm2-modal-actions pm2-modal-actions--foot">
            <button type="button" class="pm2-btn secondary" @click="m.onPreflightCancel">
              {{ t("common.cancel") }}
            </button>
            <button type="button" class="pm2-btn primary" @click="m.onPreflightConfirm">
              {{ t("pluginMarketV2.preflight.confirmAndContinue") }}
            </button>
          </div>
        </div>
      </div>
      <div
        v-if="m.permConsentVisible.value"
        class="pm2-modal-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="String(t('pluginMarketV2.permConsent.dialogLabel'))"
        @click.self="m.onPermConsentCancel"
      >
        <div class="pm2-modal" @click.stop>
          <div class="pm2-modal-h">{{ m.permConsentTitle }}</div>
          <p v-if="m.permConsentTrustSummary" class="pm2-trust-summary">
            <span class="pm2-trust-h">信任摘要</span>
            <br />
            <span class="pm2-trust-mono" style="white-space: pre-wrap">{{
              m.permConsentTrustSummary
            }}</span>
          </p>
          <p class="pm2-hint">
            {{ t("pluginMarketV2.permConsent.hint") }}
          </p>
          <p v-if="m.permTokenInfoLoading.value" class="pm2-muted" style="margin: 6px 0 0">
            {{ t("pluginMarketV2.permConsent.loadingTokenInfo") }}
          </p>
          <div class="pm2-modal-actions">
            <button type="button" class="pm2-btn secondary pm2-btn--sm" @click="m.setPermConsentAll(true)">
              {{ t("pluginMarketV2.permConsent.selectAll") }}
            </button>
            <button type="button" class="pm2-btn secondary pm2-btn--sm" @click="m.setPermConsentAll(false)">
              {{ t("pluginMarketV2.permConsent.selectNone") }}
            </button>
          </div>
          <ul class="pm2-perm-list">
            <li v-for="p in m.permConsentPerms" :key="p" class="pm2-perm-li">
              <label class="pm2-perm-row">
                <input
                  type="checkbox"
                  :checked="m.permConsentSelected[p] === true"
                  @change="
                    m.permConsentSelected = {
                      ...m.permConsentSelected,
                      [p]: ($event.target as HTMLInputElement).checked,
                    }
                  "
                />
                <span class="pm2-perm-token">{{ p }}</span>
                <span
                  v-if="m.permTokenInfoMap.get(p)?.risk"
                  class="pm2-perm-risk"
                  :class="m.riskClass(m.permTokenInfoMap.get(p)?.risk)"
                >
                  {{ m.riskLabel(m.permTokenInfoMap.get(p)?.risk) }}
                </span>
              </label>
              <div
                v-if="m.permTokenInfoMap.get(p)?.title || m.permTokenInfoMap.get(p)?.description"
                class="pm2-perm-desc"
              >
                <div v-if="m.permTokenInfoMap.get(p)?.title" class="pm2-perm-title">
                  {{ m.permTokenInfoMap.get(p)?.title }}
                </div>
                <div v-if="m.permTokenInfoMap.get(p)?.description" class="pm2-muted">
                  {{ m.permTokenInfoMap.get(p)?.description }}
                </div>
              </div>
            </li>
          </ul>
          <div class="pm2-modal-actions pm2-modal-actions--foot">
            <button type="button" class="pm2-btn secondary" @click="m.onPermConsentCancel">
              {{ t("common.cancel") }}
            </button>
            <button type="button" class="pm2-btn" @click="m.onPermConsentConfirm">{{ t("pluginMarketV2.permConsent.continueInstall") }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <section id="pm-v2-community-index" class="pm2-section">
      <p class="pm2-muted pm2-lead">
        {{ t("pluginMarketV2.lead") }}
      </p>
      <div class="pm2-section-head">
        <h3 class="pm2-h3">{{ t("pluginMarketV2.communityIndex.title") }}</h3>
        <div class="pm2-section-actions">
          <div class="pm2-market-tabs" role="tablist" :aria-label="String(t('pluginMarketV2.communityIndex.entryTypeLabel'))">
            <button
              type="button"
              class="pm2-tab pm2-tab--sm"
              :class="{ 'pm2-tab--active': m.marketEntryTab === 'plugin' }"
              role="tab"
              :aria-selected="m.marketEntryTab === 'plugin'"
              @click="m.marketEntryTab = 'plugin'"
            >
              {{ t("pluginMarketV2.tabs.plugin") }}
            </button>
            <button
              type="button"
              class="pm2-tab pm2-tab--sm"
              :class="{ 'pm2-tab--active': m.marketEntryTab === 'module' }"
              role="tab"
              :aria-selected="m.marketEntryTab === 'module'"
              @click="m.marketEntryTab = 'module'"
            >
              {{ t("pluginMarketV2.tabs.module") }}
            </button>
            <button
              type="button"
              class="pm2-tab pm2-tab--sm"
              :class="{ 'pm2-tab--active': m.marketEntryTab === 'profile' }"
              role="tab"
              :aria-selected="m.marketEntryTab === 'profile'"
              @click="m.marketEntryTab = 'profile'"
            >
              {{ t("pluginMarketV2.tabs.profile") }}
            </button>
          </div>
          <select
            class="pm2-select pm2-select--sm"
            :value="m.marketSourceSelected"
            @change="m.marketSourceSelected = ($event.target as HTMLSelectElement).value"
          >
            <option value="official">{{ t("pluginMarketV2.sources.official") }}</option>
            <option v-for="s in m.marketSources" :key="s" :value="s">{{ t("pluginMarketV2.sources.thirdParty", { s }) }}</option>
          </select>
          <button
            type="button"
            class="pm2-btn secondary pm2-btn--sm"
            :disabled="m.pluginStore.pluginMarketSyncing"
            @click="m.onSyncMarketIndex"
          >
            {{ m.pluginStore.pluginMarketSyncing ? t("pluginMarketV2.sync.syncing") : t("pluginMarketV2.sync.syncOnlineIndex") }}
          </button>
        </div>
      </div>
      <p v-if="m.pluginStore.pluginMarketError" class="pm2-err">{{ m.pluginStore.pluginMarketError }}</p>
      <p v-else-if="m.pluginStore.pluginMarketSnapshot?.warning" class="pm2-hint">
        {{ m.pluginStore.pluginMarketSnapshot.warning }}
      </p>
      <p v-if="m.pluginStore.pluginMarketSnapshot?.offlineMode" class="pm2-hint">
        {{ t("pluginMarketV2.offlineMode") }}
      </p>
      <p v-if="m.marketSourceSelected !== 'official'" class="pm2-err">
        {{ t("pluginMarketV2.thirdPartyWarning") }}
      </p>
      <p
        v-if="!m.pluginStore.pluginMarketSnapshot?.plugins?.length && !m.pluginStore.pluginMarketError"
        class="pm2-muted"
      >
        {{ t("pluginMarketV2.emptyHint") }}
      </p>
      <div
        v-else-if="m.marketRowsFiltered.length > 0"
        class="pm2-market-pager"
        role="toolbar"
        :aria-label="String(t('pluginMarketV2.pager.toolbarLabel'))"
      >
        <span class="pm2-muted">
          {{ t("pluginMarketV2.pager.summary", { total: m.marketRowsFiltered.length, page: m.marketPage, pages: m.marketTotalPages }) }}
        </span>
        <label class="pm2-muted">
          {{ t("pluginMarketV2.pager.pageSize") }}
          <select
            v-model.number="m.marketPageSize"
            class="pm2-select pm2-select--sm"
            :aria-label="String(t('pluginMarketV2.pager.pageSizeAria'))"
          >
            <option :value="15">15</option>
            <option :value="30">30</option>
            <option :value="60">60</option>
          </select>
        </label>
        <button
          type="button"
          class="pm2-btn secondary pm2-btn--sm"
          :disabled="m.marketPage <= 1"
          @click="m.marketPage = Math.max(1, m.marketPage - 1)"
        >
          {{ t("pluginMarketV2.pager.prev") }}
        </button>
        <button
          type="button"
          class="pm2-btn secondary pm2-btn--sm"
          :disabled="m.marketPage >= m.marketTotalPages"
          @click="m.marketPage = Math.min(m.marketTotalPages, m.marketPage + 1)"
        >
          {{ t("pluginMarketV2.pager.next") }}
        </button>
      </div>
      <ul v-if="m.marketRowsPaged.length > 0" class="pm2-market-list">
        <li v-for="row in m.marketRowsPaged" :key="row.id" class="pm2-market-li">
          <div class="pm2-market-main">
            <strong>{{ row.id }}</strong>
            <span
              class="pm2-source-badge"
              :class="(row.source ?? '') === 'official' ? 'official' : 'third'"
              :title="(row.source ?? '') === 'official' ? '官方默认索引' : '第三方索引源'"
            >
              {{ (row.source ?? "") === "official" ? "官方" : "第三方" }}
            </span>
            <span
              v-if="m.marketEntryType(row) !== 'plugin'"
              class="pm2-entry-type-badge"
              :class="m.marketEntryType(row)"
              :title="m.marketEntryType(row) === 'module' ? '无代码模块条目' : '无代码 Profile 条目'"
            >
              {{ m.marketEntryType(row) === "module" ? "模块" : "Profile" }}
            </span>
            <span class="pm2-muted"> · {{ row.name }} · v{{ row.version }}</span>
            <p v-if="row.source || row.publisher" class="pm2-market-trust">
              <span v-if="row.source" class="pm2-muted">来源：{{ row.source }}</span>
              <span v-if="row.publisher" class="pm2-muted"> · 发布者：{{ row.publisher }}</span>
              <span v-if="(row.publicKeys ?? []).length" class="pm2-muted" title="索引登记的公钥状态">
                · 公钥：{{
                  (row.publicKeys ?? [])
                    .map((k) => `${k.pubkeyId}${k.status ? `(${k.status})` : ""}`)
                    .join("，")
                }}
              </span>
            </p>
            <p class="pm2-market-rating">
              <span
                class="pm2-rating-stars"
                :title="`公开评价（总体）：${m.ratingTextForPluginId(row.id)}`"
              >
                {{ m.ratingStarsForPluginId(row.id) }}
              </span>
              <span class="pm2-muted"> · {{ m.ratingTextForPluginId(row.id) }}</span>
              <template v-if="(row.publicKeys ?? []).length">
                <span class="pm2-muted"> · 公钥口径：</span>
                <span
                  v-for="k in row.publicKeys ?? []"
                  :key="`rv-${row.id}-${k.pubkeyId}`"
                  class="pm2-pubkey-rating"
                  :title="`pubkeyId=${k.pubkeyId}${k.status ? ` (${k.status})` : ''}`"
                >
                  <span class="pm2-muted">{{ k.pubkeyId }}</span>
                  <span class="pm2-rating-stars">{{
                    m.ratingStarsForPluginPubkey(row.id, k.pubkeyId)
                  }}</span>
                  <span class="pm2-muted">({{ m.ratingTextForPluginPubkey(row.id, k.pubkeyId) }})</span>
                  <button
                    type="button"
                    class="pm2-link pm2-link--tiny"
                    :disabled="m.pluginReviewsLoading"
                    title="复制绑定该 pubkeyId 的评价 JSON 模板"
                    @click="
                      m.copyReviewTemplate({
                        pluginId: row.id,
                        pubkeyId: k.pubkeyId,
                        version: m.marketPickedVersionForRow(row) ?? null,
                      })
                    "
                  >
                    复制
                  </button>
                </span>
              </template>
              <button
                type="button"
                class="pm2-link"
                :disabled="m.pluginReviewsLoading"
                @click="m.openPluginReviewsContribution"
              >
                去提交评价
              </button>
              <button
                type="button"
                class="pm2-link"
                :disabled="m.pluginReviewsLoading"
                @click="
                  m.copyReviewTemplate({
                    pluginId: row.id,
                    pubkeyId: row.publicKeys?.[0]?.pubkeyId ?? null,
                    version: m.marketPickedVersionForRow(row) ?? null,
                  })
                "
                title="复制一段可直接提交到 reviews.json 的 JSON 模板（建议按 pubkeyId 口径提交）"
              >
                复制模板
              </button>
              <button
                type="button"
                class="pm2-link"
                :disabled="m.pluginReviewsLoading"
                @click="m.syncPluginReviewsIndexNow"
              >
                刷新评价
              </button>
              <span v-if="m.pluginReviewsErr" class="pm2-err"> · {{ m.pluginReviewsErr }}</span>
            </p>
            <div
              v-if="
                m.getRecentReviews(m.pluginReviewsIndex?.reviews ?? [], {
                  pluginId: row.id,
                  limit: 3,
                }).length
              "
              class="pm2-market-reviews"
            >
              <p class="pm2-market-reviews-h">最近短评：</p>
              <ul class="pm2-market-reviews-list">
                <li
                  v-for="r in m.getRecentReviews(m.pluginReviewsIndex?.reviews ?? [], {
                    pluginId: row.id,
                    limit: 3,
                  })"
                  :key="`rr-${row.id}-${r.id}`"
                >
                  <span class="pm2-market-review-line" :title="r.created_at">{{
                    m.renderReviewLine(r)
                  }}</span>
                </li>
              </ul>
            </div>
            <p v-if="row.description" class="pm2-market-desc">{{ row.description }}</p>
            <details
              v-if="m.marketEntryType(row) === 'module' && (row as any).module"
              class="pm2-market-details"
            >
              <summary class="pm2-market-details-sum">查看模块声明</summary>
              <div class="pm2-market-details-body">
                <p
                  v-if="(((row as any).module.plugins ?? []) as any[]).length"
                  class="pm2-muted"
                >
                  依赖插件：{{
                    ((row as any).module.plugins ?? []).map((x: any) => x.id).join("、")
                  }}
                </p>
                <div
                  v-if="
                    m.summarizeOverrideBackends(((row as any).module.backends ?? null) as any).length
                  "
                >
                  <p class="pm2-muted">后端覆盖（会话级）：</p>
                  <ul class="pm2-kv-list">
                    <li
                      v-for="(x, idx) in m.summarizeOverrideBackends(
                        ((row as any).module.backends ?? null) as any,
                      )"
                      :key="`mb-${idx}`"
                      class="pm2-kv-li"
                    >
                      {{ x }}
                    </li>
                  </ul>
                </div>
                <p v-else class="pm2-muted">未声明 backends 覆盖。</p>
              </div>
            </details>
            <details
              v-else-if="m.marketEntryType(row) === 'profile' && (row as any).profile"
              class="pm2-market-details"
            >
              <summary class="pm2-market-details-sum">查看 Profile 声明</summary>
              <div class="pm2-market-details-body">
                <p
                  v-if="(((row as any).profile.plugins ?? []) as any[]).length"
                  class="pm2-muted"
                >
                  依赖插件：{{
                    ((row as any).profile.plugins ?? []).map((x: any) => x.id).join("、")
                  }}
                </p>
                <p
                  v-if="(((row as any).profile.predeclaredPermissions ?? []) as any[]).length"
                  class="pm2-muted"
                >
                  预声明权限：{{
                    ((row as any).profile.predeclaredPermissions ?? []).join("、")
                  }}
                </p>
                <div
                  v-if="
                    m.summarizeOverrideBackends(((row as any).profile.backends ?? null) as any).length
                  "
                >
                  <p class="pm2-muted">后端覆盖（会话级）：</p>
                  <ul class="pm2-kv-list">
                    <li
                      v-for="(x, idx) in m.summarizeOverrideBackends(
                        ((row as any).profile.backends ?? null) as any,
                      )"
                      :key="`pb-${idx}`"
                      class="pm2-kv-li"
                    >
                      {{ x }}
                    </li>
                  </ul>
                </div>
                <p v-else class="pm2-muted">未声明 backends 覆盖。</p>
              </div>
            </details>
            <p v-if="(row.missingDependencies ?? []).length" class="pm2-err pm2-market-deps">
              依赖缺失：{{ row.missingDependencies.join("、") }}
            </p>
          </div>
          <div class="pm2-market-actions">
            <button
              v-if="m.marketEntryType(row) === 'module'"
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="m.onApplyModuleEntry(row)"
            >
              应用模块
            </button>
            <button
              v-else-if="m.marketEntryType(row) === 'profile'"
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="m.onApplyProfileEntry(row)"
            >
              应用 Profile
            </button>
            <div v-else-if="(row.versions ?? []).length > 0" class="pm2-market-versions">
              <select
                class="pm2-select pm2-select--sm"
                :value="m.marketPickedVersionForRow(row)"
                @change="
                  m.marketPickedVersion = {
                    ...m.marketPickedVersion,
                    [row.id]: ($event.target as HTMLSelectElement).value,
                  }
                "
              >
                <option v-for="v in m.marketVersionsForRow(row)" :key="`${row.id}-${v}`" :value="v">
                  v{{ v }}
                </option>
              </select>
              <button
                type="button"
                class="pm2-btn secondary pm2-btn--sm"
                @click="m.onInstallMarketVersion(row)"
              >
                {{ row.installed ? "回滚/切换" : "安装此版本" }}
              </button>
            </div>
            <button
              v-else-if="!row.installed"
              type="button"
              class="pm2-btn secondary pm2-btn--sm"
              @click="m.onInstallMarketEntry(row)"
            >
              安装
            </button>
            <template v-else>
              <span v-if="row.hasUpdate" class="pm2-badge">可更新</span>
              <span v-else class="pm2-muted">已安装</span>
              <button
                v-if="row.hasUpdate"
                type="button"
                class="pm2-btn secondary pm2-btn--sm"
                @click="m.onUpdateMarketEntry(row)"
              >
                更新
              </button>
            </template>
          </div>
        </li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.pm2-market-wrap {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: auto;
  padding-right: 4px;
}
.pm2-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10090;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--dialog-backdrop, color-mix(in srgb, #000 55%, transparent));
}
.pm2-modal {
  width: min(520px, 100%);
  max-height: min(86vh, 720px);
  overflow: auto;
  padding: 14px 16px 12px;
  border-radius: var(--radius-app);
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  box-shadow: var(--shadow-app);
}
.pm2-modal-h {
  font-size: 14px;
  font-weight: 600;
  margin: 0 0 8px;
}
.pm2-trust-summary {
  margin: 0 0 10px;
  padding: 8px 10px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-trust-h {
  font-weight: 600;
  color: var(--text-secondary);
}
.pm2-trust-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm2-modal-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin: 8px 0;
}
.pm2-modal-actions--foot {
  justify-content: flex-end;
  margin-top: 12px;
}
.pm2-perm-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm2-preflight-list {
  list-style: none;
  padding: 0;
  margin: 10px 0 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.pm2-preflight-li {
  margin: 0;
  padding: 8px 10px;
  border: 1px solid var(--border-light);
  border-radius: 10px;
  background: var(--bg-secondary);
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-perm-li {
  margin: 0;
}
.pm2-perm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}
.pm2-perm-token {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
}
.pm2-perm-title {
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-perm-desc {
  margin-left: 22px;
  margin-top: 2px;
}
.pm2-perm-risk {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
}
.pm2-perm-risk.risk-high {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
}
.pm2-perm-risk.risk-medium {
  color: var(--warn-700, #b9770e);
  border-color: color-mix(in srgb, var(--warn-700, #b9770e) 40%, var(--border-light));
}
.pm2-perm-risk.risk-low {
  color: var(--success-700, #1e7e34);
  border-color: color-mix(in srgb, var(--success-700, #1e7e34) 40%, var(--border-light));
}
.pm2-hint {
  margin: 0 0 8px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}
.pm2-muted {
  color: var(--text-secondary);
}
.pm2-err {
  margin: 4px 0;
  font-size: 13px;
  color: var(--danger-600, #c0392b);
}
.pm2-lead {
  margin: 0 0 12px;
  font-size: 12px;
  line-height: 1.45;
}
.pm2-section {
  margin-bottom: 8px;
  padding: 14px 16px;
  border-radius: 14px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
}
.pm2-section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}
.pm2-h3 {
  margin: 0;
  font-size: 19px;
  line-height: 1.35;
}
.pm2-section-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}
.pm2-market-tabs {
  display: flex;
  gap: 6px;
  align-items: center;
}
.pm2-tab {
  flex: 0 0 auto;
  padding: 8px 12px;
  border: 1px solid transparent;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  color: var(--text-secondary);
  background: transparent;
}
.pm2-tab--sm {
  padding: 8px 12px;
  font-size: 13px;
}
.pm2-tab--active {
  color: var(--text-primary);
  border-color: var(--border-light);
  background: var(--bg-elevated);
}
.pm2-select {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}
.pm2-select--sm {
  padding: 5px 8px;
  font-size: 12px;
}
.pm2-btn {
  padding: 7px 12px;
  border-radius: 8px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  cursor: pointer;
  font-size: 13px;
}
.pm2-btn.secondary {
  background: transparent;
}
.pm2-btn.primary {
  background: color-mix(in srgb, var(--accent) 18%, var(--bg-elevated));
  border-color: color-mix(in srgb, var(--accent) 35%, var(--border-light));
}
.pm2-btn--sm {
  padding: 5px 10px;
  font-size: 12px;
}
.pm2-market-pager {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  margin: 8px 0 10px;
}
.pm2-market-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: min(440px, 52vh);
  overflow: auto;
}
.pm2-market-li {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--border-light);
  background: var(--bg-secondary);
  font-size: 13px;
}
.pm2-market-main {
  flex: 1 1 200px;
  min-width: 0;
}
.pm2-market-desc {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm2-market-trust {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
}
.pm2-market-rating {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.45;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}
.pm2-rating-stars {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
    "Courier New", monospace;
  letter-spacing: 0.5px;
}
.pm2-link {
  border: none;
  background: none;
  padding: 0;
  font: inherit;
  color: var(--accent, #6b8cff);
  text-decoration: underline;
  cursor: pointer;
}
.pm2-link:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  text-decoration: none;
}
.pm2-link--tiny {
  font-size: 11px;
  text-decoration: none;
  border: 1px solid var(--border-light);
  border-radius: 8px;
  padding: 1px 6px;
}
.pm2-pubkey-rating {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 1px 6px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
}
.pm2-source-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  font-size: 11px;
  font-weight: 600;
  vertical-align: middle;
}
.pm2-source-badge.official {
  color: var(--success-700, #1e7e34);
  border-color: color-mix(in srgb, var(--success-700, #1e7e34) 40%, var(--border-light));
  background: color-mix(in srgb, var(--success-700, #1e7e34) 8%, var(--bg-primary));
}
.pm2-source-badge.third {
  color: var(--danger-600, #c0392b);
  border-color: color-mix(in srgb, var(--danger-600, #c0392b) 40%, var(--border-light));
  background: color-mix(in srgb, var(--danger-600, #c0392b) 8%, var(--bg-primary));
}
.pm2-market-reviews {
  margin: 4px 0 0;
  padding: 6px 8px;
  border-radius: 10px;
  border: 1px dashed var(--border-light);
  background: color-mix(in srgb, var(--bg-primary) 70%, transparent);
}
.pm2-market-reviews-h {
  margin: 0 0 4px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-market-reviews-list {
  margin: 0;
  padding-left: 16px;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-market-review-line {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: inline-block;
  max-width: min(860px, 86vw);
  vertical-align: bottom;
}
.pm2-market-deps {
  margin: 6px 0 0;
  font-size: 12px;
}
.pm2-market-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm2-market-versions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.pm2-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: color-mix(in srgb, var(--warn-700, #b9770e) 12%, var(--bg-elevated));
  color: var(--text-secondary);
}
.pm2-entry-type-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 2px 6px;
  font-size: 11px;
  border-radius: 999px;
  border: 1px solid var(--border-light);
  background: var(--bg-elevated);
  color: var(--text-secondary);
}
.pm2-entry-type-badge.module {
  border-color: color-mix(in srgb, var(--border-light) 70%, #4f46e5);
}
.pm2-entry-type-badge.profile {
  border-color: color-mix(in srgb, var(--border-light) 70%, #16a34a);
}
.pm2-market-details {
  margin-top: 8px;
}
.pm2-market-details-sum {
  cursor: pointer;
  user-select: none;
  font-size: 12px;
  color: var(--text-secondary);
}
.pm2-market-details-body {
  margin-top: 6px;
}
.pm2-kv-list {
  list-style: none;
  padding: 0;
  margin: 6px 0 0;
}
.pm2-kv-li {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 2px 0;
}
</style>
