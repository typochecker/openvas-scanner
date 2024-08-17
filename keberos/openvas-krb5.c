#include "openvas-krb5.h"

#include <assert.h>
#include <krb5/krb5.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GUARD_NULL(var, return_var)          \
  do                                         \
    {                                        \
      if (var != NULL)                       \
        {                                    \
          return_var = O_KRB5_EXPECTED_NULL; \
          goto result;                       \
        }                                    \
    }                                        \
  while (0)

#define GUARD_NOT_NULL(var, return_var)          \
  do                                             \
    {                                            \
      if (var == NULL)                           \
        {                                        \
          return_var = O_KRB5_EXPECTED_NOT_NULL; \
          goto result;                           \
        }                                        \
    }                                            \
  while (0)

#define ALLOCATE_AND_CHECK(var, type, n, return_var) \
  do                                                 \
    {                                                \
      var = (type *) calloc (n, sizeof (type));      \
      if (var == NULL)                               \
        {                                            \
          return_var = O_KRB5_NOMEM;                 \
          goto result;                               \
        }                                            \
    }                                                \
  while (0)

#define SKIP_WS(line, line_len, start, i)        \
  do                                             \
    {                                            \
      for (i = start; i < line_len; i++)         \
        {                                        \
          if (line[i] != ' ' && line[i] != '\t') \
            {                                    \
              break;                             \
            }                                    \
        }                                        \
    }                                            \
  while (0)

#define IS_STR_EQUAL(line, line_len, start, cmp, cmp_len) \
  ((line_len - start < cmp_len) ? 0                       \
   : (line_len == 0 && cmp_len == 0)                      \
     ? 1                                                  \
     : (memcmp (line + start, cmp, cmp_len) == 0))

#define MAX_LINE_LENGTH 1024
// Finds the kdc defined for the given realm.
OKrb5ErrorCode
o_krb5_find_kdc (const OKrb5Credential *creds, char **kdc)
{
  OKrb5ErrorCode result = O_KRB5_REALM_NOT_FOUND;
  char line[MAX_LINE_LENGTH];
  int state = 0;
  int last_element;
  int i, j;
  FILE *file;

  int realm_len = strlen (creds->realm);

  // we don't know if we should free it or just override it.
  // aborting instead.
  GUARD_NULL (*kdc, result);
  if ((file = fopen (creds->config_path, "r")) == NULL)
    {
      result = O_KRB5_CONF_NOT_FOUND;
      goto result;
    }

  while (fgets (line, MAX_LINE_LENGTH, file))
    {
      line[strcspn (line, "\n")] = 0;
      last_element = strlen (line) - 1;
      SKIP_WS (line, last_element, 0, i);
      if (line[i] == '[' && line[last_element] == ']')
        {
          if (state != 0)
            {
              result = O_KRB5_REALM_NOT_FOUND;
              goto result;
            }
          if (IS_STR_EQUAL (line, last_element + 1, i, "[realms]", 8) == 1)
            {
              state = 1;
            }
        }
      else
        {
          if (line[i] == '}' || line[last_element] == '}')
            {
              state = 1;
            }
          else if (state == 1)
            {
              for (j = i; j <= last_element; j++)
                {
                  if (line[j] != creds->realm[j - i])
                    {
                      state = 2;
                      break;
                    }
                  if (j - i >= realm_len)
                    {
                      break;
                    }
                }
              if (j - i == realm_len)
                {
                  state = 3;
                }
            }
          else if (state == 3)
            {
              if (IS_STR_EQUAL (line, last_element + 1, i, "kdc", 3))
                {
                  SKIP_WS (line, last_element, i + 3, i);
                  if (line[i] == '=')
                    {
                      SKIP_WS (line, last_element, i + 1, i);
                      ALLOCATE_AND_CHECK (*kdc, char, (last_element - i) + 1,
                                          result);
                      for (j = i; j <= last_element; j++)
                        {
                          (*kdc)[j - i] = line[j];
                        }

                      result = O_KRB5_SUCCESS;
                      goto result;
                    }
                }
            }
        }
    }

result:
  if (result != O_KRB5_CONF_NOT_FOUND)
    {
      fclose (file);
    }
  return result;
}
// Adds realm with the given kdc into krb5.conf
OKrb5ErrorCode
o_krb5_add_realm (const OKrb5Credential *creds, const char *kdc)
{
  OKrb5ErrorCode result = O_KRB5_SUCCESS;
  FILE *file = NULL, *tmp = NULL;
  char line[MAX_LINE_LENGTH] = {0};
  char tmpfn[MAX_LINE_LENGTH] = {0};
  int state, i;
  // TODO: validate output of fprintf when writing and abort

  if ((file = fopen (creds->config_path, "r")) == NULL)
    {
      if ((file = fopen (creds->config_path, "w")) == NULL)
        {
          result = O_KRB5_CONF_NOT_CREATED;
          goto result;
        }
      fprintf (file, "[realms]\n%s = {\n  kdc = %s\n}\n", creds->realm, kdc);
      goto close_config;
    }
  snprintf (tmpfn, MAX_LINE_LENGTH, "%s.tmp", creds->config_path);
  if ((tmp = fopen (tmpfn, "w")) == NULL)
    {
      result = O_KRB5_TMP_CONF_NOT_CREATED;
      goto close_config;
    }
  state = 0;
  while (fgets (line, MAX_LINE_LENGTH, file))
    {
      fputs (line, tmp);
      if (state == 0)
        {
          SKIP_WS (line, MAX_LINE_LENGTH, 0, i);
          if (IS_STR_EQUAL (line, MAX_LINE_LENGTH, i, "[realms]", 8) == 1)
            {
              fprintf (tmp, "%s = {\n  kdc = %s\n}\n", creds->realm, kdc);
              state = 1;
            }
        }
    }

  if (rename (tmpfn, creds->config_path) != 0)
    {
      result = O_KRB5_TMP_CONF_NOT_MOVED;
    }

close_tmp:
  fclose (tmp);
close_config:
  fclose (file);
result:
  return result;
}

OKrb5ErrorCode
o_krb5_authenticate (const OKrb5Credential credentials, OKrb5Element **element)
{
  OKrb5ErrorCode result = O_KRB5_SUCCESS;

  // probably better to use heap instead?
  krb5_context ctx;
  krb5_principal me;
  krb5_creds creds;

  GUARD_NULL (*element, result);
  if ((result = krb5_init_context (&ctx)))
    {
      result = result + O_KRB5_ERROR;
      goto result;
    }

  if ((result =
         krb5_build_principal (ctx, &me, strlen (credentials.realm),
                               credentials.realm, credentials.user, NULL)))
    {
      result = result + O_KRB5_ERROR;
      goto result;
    };

  if ((result = krb5_get_init_creds_password (
         ctx, &creds, me, credentials.password, NULL, NULL, 0, NULL, NULL)))
    {
      result = result + O_KRB5_ERROR;
      goto result;
    }
  ALLOCATE_AND_CHECK (*element, OKrb5Element, 1, result);
  (*element)->me = me;
  (*element)->creds = creds;
  (*element)->ctx = ctx;

result:
  if (result != O_KRB5_SUCCESS)
    {
      krb5_free_cred_contents (ctx, &creds);
      krb5_free_principal (ctx, me);
      krb5_free_context (ctx);
      if (*element != NULL)
        {
          free (*element);
          *element = NULL;
        }
    }

  return result;
}

OKrb5ErrorCode
o_krb5_free_element (OKrb5Element *element)
{
  OKrb5ErrorCode result = O_KRB5_SUCCESS;

  if (element != NULL)
    {
      krb5_free_cred_contents (element->ctx, &element->creds);
      krb5_free_principal (element->ctx, element->me);
      krb5_free_context (element->ctx);
      free (element);
    }
  return result;
}

OKrb5ErrorCode
o_krb5_request (const OKrb5Element *element, const char *data,
                const size_t data_len, OKrb5Data **out)
{
  OKrb5ErrorCode result = O_KRB5_SUCCESS;
  GUARD_NOT_NULL (out, result);
  GUARD_NULL (*out, result);
  GUARD_NOT_NULL (element, result);
  ALLOCATE_AND_CHECK (*out, OKrb5Data, 1, result);
  krb5_data in_data;
  int ap_req_options = 0;

  in_data.length = data_len;
  in_data.data = (char *) data;

  if ((result = krb5_auth_con_init (element->ctx, &(*out)->auth_context)))
    {
      result = result + O_KRB5_ERROR;
      goto result;
    };

  if ((result = krb5_mk_req_extended (
         element->ctx, &(*out)->auth_context, ap_req_options, &in_data,
         (krb5_creds *) &(element->creds), &(*out)->data)))
    {
      result = result + O_KRB5_ERROR;
    };
result:
  return result;
}

OKrb5ErrorCode
o_krb5_free_data (const OKrb5Element *element, OKrb5Data *data)
{
  OKrb5ErrorCode result = O_KRB5_SUCCESS;
  GUARD_NOT_NULL (element, result);

  if (data != NULL)
    {
      if ((result = krb5_auth_con_free (element->ctx, data->auth_context)))
        {
          result += O_KRB5_ERROR;
          goto result;
        };
      free (data);
    }
result:
  return result;
}